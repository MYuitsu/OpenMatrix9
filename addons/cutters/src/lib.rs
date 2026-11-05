// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix-style **Cutters** menu commands for OM9 (besides Gem Cutter, in `gem_cutter`):
//!
//! - **Gem Cutter Library**: the gem cutter in ready styles (setting, azure, straight, brite);
//! - **Bezel Cutter**: scallop cutters around a stone's bezel;
//! - **Channel Builder** / **Bright Cut Channel**: one cutter along a line of stones;
//! - **Micro Prong Cutter**: notches between neighbouring stones, leaving micro prongs;
//! - **Bright Cutter**: V grooves along both sides of a line of stones;
//! - **Plane and Cube Cutters**: boxes clicked onto the CPlane;
//! - **Cut to Finger Rail**: the finger cylinder of a ring rail, to subtract;
//! - **Quad Flip**: mirror the design into the four CPlane quadrants.
//!
//! Cutters are closed meshes on the `Cutting` layer; the stones and metal are not changed.
use gem_cutter::Params as CutterParams;
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, OptionToken, Reply, TransformOp, Vec3, Want, add, cross, dist,
    fmt, mirror, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{
    Built, ignored, load, save, selected_curves, selected_gems, selection_command,
};
use om9_addon_geom::gems::{PlacedGem, chain};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const LIBRARY: &str = "BuilderGemCutterLibrary";
pub const BEZEL_CUTTER: &str = "BuilderBezelCutter";
pub const CHANNEL: &str = "BuilderChannelBuilder";
pub const BRIGHT_CHANNEL: &str = "BuilderBrightCutChannel";
pub const MICRO_PRONG: &str = "BuilderMicroProngCutter";
pub const BRIGHT_CUTTER: &str = "BuilderBrightCutter";
pub const PLANE_CUBE: &str = "BuilderPlaneandCubeCutters";
pub const FINGER_RAIL: &str = "OthersCuttoFingerRail";
pub const QUAD_FLIP: &str = "BuilderQuadFlip";

pub const CUTTING: &str = "Cutting";

pub struct Cutters;

impl Interactive for Cutters {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        LIBRARY,
        BEZEL_CUTTER,
        CHANNEL,
        BRIGHT_CHANNEL,
        MICRO_PRONG,
        BRIGHT_CUTTER,
        PLANE_CUBE,
        FINGER_RAIL,
        QUAD_FLIP,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            LIBRARY => library(e),
            BEZEL_CUTTER => bezel_cutter(e),
            CHANNEL => channel(e, false),
            BRIGHT_CHANNEL => channel(e, true),
            MICRO_PRONG => micro_prong(e),
            BRIGHT_CUTTER => bright_cutter(e),
            PLANE_CUBE => plane_cube(e),
            FINGER_RAIL => finger_rail(e),
            QUAD_FLIP => quad_flip(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

fn cutter(m: Mesh, name: &str) -> Geo {
    m.geo(name, None, Some(CUTTING))
}

// ---- Gem Cutter Library ----------------------------------------------------------------------

/// The gem cutter of a library style, scaled across (xy) and along (z) the stone axis.
pub fn library_params(style: &str, xy: f64, z: f64) -> CutterParams {
    let mut p = CutterParams::default();
    match style {
        "azure" => {
            p.seat_size = 60.0;
            p.lower_depth = 260.0;
            p.top_height = 60.0;
        }
        "straight" => {
            p.seat_size = 100.0;
            p.lower_depth = 120.0;
        }
        "brite" => {
            p.girdle_scale = 115.0;
            p.top_height = 80.0;
            p.seat_size = 45.0;
        }
        _ => {}
    }
    p.girdle_scale *= xy;
    p.top_height *= z;
    p.lower_depth *= z;
    p
}

fn library(e: &Event) -> Reply {
    let style = e.text("lib_style", "setting").to_owned();
    let p = library_params(
        &style,
        e.number("lib_scale_xy", 100.0) / 100.0,
        e.number("lib_scale_z", 100.0) / 100.0,
    );
    selection_command(e, "Gem Cutter Library", "select stones, Enter to create the cutters", |e| {
        let (gems, other) = selected_gems(e);
        let create: Vec<Geo> = gems.iter().map(|g| gem_cutter::cutter_for(g, &p)).collect();
        Ok(Built {
            summary: format!("{} {style} cutters{}", create.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Bezel Cutter (scallops) -----------------------------------------------------------------

pub fn bezel_cutters_for(g: &PlacedGem, count: usize, size: f64, height: f64, length: f64, angle: f64) -> Vec<Mesh> {
    let r = g.gem.base() / 2.0;
    let reach = if length > 0.0 { length } else { r + 1.5 };
    (0..count)
        .map(|k| {
            let a = (angle + 360.0 * k as f64 / count as f64).to_radians();
            let dir = g.frame.vector([a.cos(), a.sin(), 0.0]);
            let up = g.frame.z;
            // A cylinder lying radially, its axis `height` above the girdle.
            let start = add(g.frame.point([0.0, 0.0, height]), scale(dir, r * 0.5));
            let local = mesh::cylinder(size / 2.0, 0.0, reach - r * 0.5, 24);
            local.placed(&Frame::new(start, dir, up))
        })
        .collect()
}

fn bezel_cutter(e: &Event) -> Reply {
    let count = e.number("bzc_count", 6.0).round().clamp(1.0, 36.0) as usize;
    let size = e.number("bzc_size", 1.0).max(0.05);
    let height = e.number("bzc_height", 0.6);
    let length = e.number("bzc_length", 0.0);
    let angle = e.number("bzc_angle", 0.0);
    selection_command(e, "Bezel Cutter", "select stones (with their bezels), Enter to create", |e| {
        let (gems, other) = selected_gems(e);
        let create: Vec<Geo> = gems
            .iter()
            .flat_map(|g| bezel_cutters_for(g, count, size, height, length, angle))
            .map(|m| cutter(m, "Bezel Cutter"))
            .collect();
        Ok(Built {
            summary: format!("{} scallop cutters Ø{} mm{}", create.len(), fmt(size), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Channel Builder / Bright Cut Channel ------------------------------------------------------

/// Path through a line of stones (centers), extended past both ends of an open line.
fn line_path(gems: &[PlacedGem], order: &[usize], closed: bool, extension: f64) -> (Vec<Vec3>, Vec<f64>, Vec<Vec3>) {
    let mut pts: Vec<Vec3> = order.iter().map(|&i| gems[i].frame.origin).collect();
    let mut sizes: Vec<f64> = order.iter().map(|&i| gems[i].gem.base()).collect();
    let mut ups: Vec<Vec3> = order.iter().map(|&i| gems[i].frame.z).collect();
    if !closed && pts.len() >= 2 {
        let n = pts.len();
        let d0 = unit(sub(pts[0], pts[1]));
        let d1 = unit(sub(pts[n - 1], pts[n - 2]));
        let first = add(pts[0], scale(d0, sizes[0] / 2.0 + extension));
        let last = add(pts[n - 1], scale(d1, sizes[n - 1] / 2.0 + extension));
        pts.insert(0, first);
        pts.push(last);
        sizes.insert(0, sizes[0]);
        sizes.push(sizes[n]);
        ups.insert(0, ups[0]);
        ups.push(ups[n]);
    }
    (pts, sizes, ups)
}

fn channel(e: &Event, bright: bool) -> Reply {
    let label = if bright { "Bright Cut Channel" } else { "Channel Builder" };
    let width = e.number(if bright { "bcc_width" } else { "ch_width" }, if bright { 130.0 } else { 100.0 }) / 100.0;
    let top = e.number("ch_top", 40.0) / 100.0;
    let bottom = e.number("ch_bottom", 70.0) / 100.0;
    let extension = e.number("ch_extension", 0.5);
    let z = e.number("ch_zoffset", 0.0);
    selection_command(e, label, "select a line of stones, Enter to create the channel cutter", |e| {
        let (gems, other) = selected_gems(e);
        if gems.len() < 2 {
            return Err(format!("{label}: select at least two stones in a line"));
        }
        let (order, closed) = chain(&gems);
        let (path, sizes, ups) = line_path(&gems, &order, closed, extension);
        let rings: Vec<Vec<Vec3>> = (0..path.len())
            .map(|i| {
                let (_, u, side) = mesh::run_frame(&path, closed, i, ups[i]);
                let d = sizes[i];
                let w = d * width / 2.0;
                let section: Vec<[f64; 2]> = if bright {
                    // V: wide at the top, down to an edge below the stones.
                    vec![[w, top * d], [0.0, -bottom * d], [-w, top * d]]
                } else {
                    vec![[w, top * d], [w, -bottom * d], [-w, -bottom * d], [-w, top * d]]
                };
                let base = add(path[i], scale(u, z));
                section
                    .iter()
                    .map(|q| add(base, add(scale(side, q[0]), scale(u, q[1]))))
                    .collect()
            })
            .collect();
        let m = mesh::loft_rings(&rings, closed).ok_or_else(|| format!("{label}: the line is too short"))?;
        Ok(Built {
            summary: format!("one cutter along {} stones{}", gems.len(), ignored(other)),
            create: vec![cutter(m, label)],
            preview: Vec::new(),
        })
    })
}

// ---- Micro Prong Cutter ----------------------------------------------------------------------

fn micro_prong(e: &Event) -> Reply {
    let width = e.number("mp_width", 30.0) / 100.0;
    let height = e.number("mp_height", 60.0) / 100.0;
    let depth = e.number("mp_depth", 80.0) / 100.0;
    let overlap = e.number("mp_overlap", 20.0) / 100.0;
    selection_command(e, "Micro Prong Cutter", "select a line of stones, Enter to create the cutters", |e| {
        let (gems, other) = selected_gems(e);
        if gems.len() < 2 {
            return Err("Micro Prong Cutter: select at least two stones in a line".into());
        }
        let (order, closed) = chain(&gems);
        let pairs = if closed { order.len() } else { order.len() - 1 };
        let mut create = Vec::new();
        for k in 0..pairs {
            let (a, b) = (&gems[order[k]], &gems[order[(k + 1) % order.len()]]);
            let d = a.gem.base().min(b.gem.base());
            let (ra, rb) = (a.gem.base() / 2.0, b.gem.base() / 2.0);
            let l = dist(a.frame.origin, b.frame.origin);
            let u = unit(sub(b.frame.origin, a.frame.origin));
            let gap = (l - ra - rb).max(0.0);
            let mid = add(a.frame.origin, scale(u, ra + gap / 2.0));
            let half_len = gap / 2.0 + overlap * d;
            let up = unit(add(a.frame.z, b.frame.z));
            let local = mesh::cuboid(
                [-half_len, -width * d / 2.0, -depth * d],
                [half_len, width * d / 2.0, height * d],
            );
            create.push(cutter(local.placed(&Frame::new(mid, up, u)), "Micro Prong Cutter"));
        }
        Ok(Built {
            summary: format!("{} notch cutters between {} stones{}", create.len(), gems.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Bright Cutter -----------------------------------------------------------------------------

fn bright_cutter(e: &Event) -> Reply {
    let offset = e.number("bc_offset", 0.1);
    let width = e.number("bc_width", 0.6).max(0.05);
    let depth = e.number("bc_depth", 0.4).max(0.05);
    let height = e.number("bc_height", 0.1);
    let extension = e.number("bc_extension", 0.3);
    selection_command(e, "Bright Cutter", "select a line of stones, Enter to create the grooves", |e| {
        let (gems, other) = selected_gems(e);
        if gems.len() < 2 {
            return Err("Bright Cutter: select at least two stones in a line".into());
        }
        let (order, closed) = chain(&gems);
        let (path, sizes, ups) = line_path(&gems, &order, closed, extension);
        let mut create = Vec::new();
        for s in [1.0, -1.0] {
            let rings: Vec<Vec<Vec3>> = (0..path.len())
                .map(|i| {
                    let (_, u, side) = mesh::run_frame(&path, closed, i, ups[i]);
                    let c = sizes[i] / 2.0 + offset + width / 2.0;
                    let pts = [[c - width / 2.0, height], [c, height - depth], [c + width / 2.0, height]];
                    pts.iter()
                        .map(|q| add(path[i], add(scale(side, s * q[0]), scale(u, q[1] + 0.0))))
                        .collect()
                })
                .collect();
            if let Some(m) = mesh::loft_rings(&rings, closed) {
                create.push(cutter(m, "Bright Cutter"));
            }
        }
        Ok(Built {
            summary: format!("{} grooves along {} stones{}", create.len(), gems.len(), ignored(other)),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Plane and Cube Cutters ------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Clicks {
    points: Vec<Vec3>,
}

fn plane_cube(e: &Event) -> Reply {
    let plane_mode = e.text("cube_mode", "cube") == "plane";
    let (x, y, z) = (
        e.number("cube_x", 5.0).max(0.01),
        e.number("cube_y", 5.0).max(0.01),
        e.number("cube_z", 5.0).max(0.01),
    );
    let mut st: Clicks = load(e);
    let make = |p: Vec3| -> Mesh {
        let local = if plane_mode {
            // A slab under the clicked point: everything below the point is cut away.
            mesh::cuboid([-x / 2.0, -y / 2.0, -z], [x / 2.0, y / 2.0, 0.0])
        } else {
            mesh::cuboid([-x / 2.0, -y / 2.0, -z / 2.0], [x / 2.0, y / 2.0, z / 2.0])
        };
        local.placed(&Frame::new(p, e.plane.normal(), e.plane.x))
    };
    let mut hover = None;
    match &e.input {
        Input::Point { point } => st.points.push(*point),
        Input::Hover { point } => hover = Some(*point),
        Input::Undo => {
            st.points.pop();
        }
        Input::Enter => {
            if st.points.is_empty() {
                st.points.push(e.plane.origin);
            }
            let name = if plane_mode { "Plane Cutter" } else { "Cube Cutter" };
            return Reply::finish(st.points.iter().map(|&p| cutter(make(p), name)).collect()).say(format!(
                "{name}: {} cutters {}×{}×{} mm",
                st.points.len(),
                fmt(x),
                fmt(y),
                fmt(z)
            ));
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = st.points.iter().flat_map(|&p| make(p).wire()).collect();
    if let Some(h) = hover {
        preview.extend(make(h).wire());
    }
    let mut options = Vec::new();
    if !st.points.is_empty() {
        options.push(OptionToken::action("Undo"));
    }
    Reply::ask(
        format!(
            "{}: click to place ({} placed), Enter to create",
            if plane_mode { "Plane Cutter" } else { "Cube Cutter" },
            st.points.len()
        ),
        save(&st),
    )
    .with_options(options)
    .want(Want::Point)
    .preview(preview)
}

// ---- Cut to Finger Rail ----------------------------------------------------------------------

/// Best circle through closed curve samples: centroid, mean radius, normal.
pub fn fit_circle(points: &[Vec3]) -> Option<(Vec3, f64, Vec3)> {
    if points.len() < 3 {
        return None;
    }
    let n = points.len() as f64;
    let c = scale(points.iter().fold([0.0; 3], |a, &p| add(a, p)), 1.0 / n);
    let r = points.iter().map(|&p| dist(p, c)).sum::<f64>() / n;
    let mut normal = [0.0; 3];
    for i in 0..points.len() {
        let a = sub(points[i], c);
        let b = sub(points[(i + 1) % points.len()], c);
        normal = add(normal, cross(a, b));
    }
    (norm(normal) > 1e-12 && r > 0.0).then(|| (c, r, unit(normal)))
}

fn finger_rail(e: &Event) -> Reply {
    let length = e.number("ftr_length", 30.0).max(1.0);
    selection_command(e, "Cut to Finger Rail", "select the ring rail (closed curve), Enter to create", |e| {
        let mut create = Vec::new();
        for c in selected_curves(e) {
            let Some((center, r, n)) = fit_circle(&c.points) else { continue };
            let local = mesh::cylinder(r, -length / 2.0, length / 2.0, 96);
            create.push(cutter(local.placed(&Frame::new(center, n, e.plane.x)), "Finger Cutter"));
        }
        Ok(Built {
            summary: format!("{} finger cylinders (subtract them to cut the inside)", create.len()),
            create,
            preview: Vec::new(),
        })
    })
}

// ---- Quad Flip -------------------------------------------------------------------------------

fn quad_flip(e: &Event) -> Reply {
    if e.selection.is_empty() || !matches!(e.input, Input::Enter) {
        let r = Reply::ask(
            format!(
                "Quad Flip: {} objects selected. Enter mirrors them into the other three CPlane quadrants",
                e.selection.len()
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if e.selection.is_empty() && matches!(e.input, Input::Enter) {
            r.refuse("Quad Flip: select the quarter of the design to repeat")
        } else {
            r
        };
    }
    let o = e.plane.origin;
    let (x, y) = (e.plane.x, e.plane.y);
    // Mirror across the plane through o with normal x (left/right), y (front/back), and both.
    let mx = mirror(o, x);
    let my = mirror(o, y);
    let mxy = om9_addon_abi::session::compose(&mx, &my);
    Reply::finish_transform(TransformOp {
        matrices: vec![mx, my, mxy],
        keep_original: true,
        per_object: Vec::new(),
    })
    .say(format!("Quad Flip: {} objects × 3 mirrored copies", e.selection.len()))
}

#[cfg(test)]
mod tests;
