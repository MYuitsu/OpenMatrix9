// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix-style **Tools** menu for OM9: ring rails and profiles (ring rail, profile
//! placer, outside rail, custom rails, four-rail profile, profile, merge and end cap), smart
//! targets and smart blends, metal weights, ring resizer, surface pullback, findings
//! libraries (our own parametric parts), image trace, object checker, mesh repair, reducer
//! and conversion, 3D printing sprues, object on curve, surface inset, curve transform and
//! center line.
//!
//! Ring rails follow the Matrix habit: the finger circle lies in the Front view (world XZ)
//! around the origin, the ring axis is world Y.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, ObjectMatrix, Replacement, Reply, Selected, SelectedMesh,
    TransformOp, Vec3, Want, add, apply, cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save, selected_gems};
use om9_addon_geom::csg::{self, Op};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use om9_addon_geom::path::{Path, Target, frame_matrix};
use om9_addon_geom::{interpolate, nurbs, section, shapes};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::f64::consts::{PI, TAU};

pub const RING_RAIL: &str = "BuilderRingRail";
pub const PROFILE_PLACER: &str = "BuilderProfilePlacer";
pub const OUTSIDE_RAIL: &str = "BuilderOutsideRingRail";
pub const CUSTOM_RAIL: &str = "BuilderCustomRail";
pub const FOUR_PROFILE: &str = "BuilderFourProfile";
pub const METAL_WEIGHTS: &str = "BuilderMetalWeights";
pub const RESIZE: &str = "BuilderResize";
pub const SURFACE_PULLBACK: &str = "BuilderSurfacePullback";
pub const SETTINGS_LIBRARY: &str = "BuilderBermarkLibrary";
pub const FINDINGS_LIBRARY: &str = "BuilderStullerFindingsLibrary";
pub const SMART_TARGET: &str = "gvSmartTarget";
pub const SMART_TARGET_ADD: &str = "gvSmartTargetAdd";
pub const SMART_TARGET_END: &str = "gvSmartTargetOnCrvEnd";
pub const SMART_BLEND: &str = "gvSmartBlend";
pub const SMART_MSR: &str = "gvSmartMSR";
pub const JOIN_HISTORY: &str = "gvJoinHistory";
pub const IMAGE_TRACE: &str = "BuilderImageTrace";
pub const OBJECT_CHECKER: &str = "BuilderObjectChecker";
pub const MESH_REPAIR: &str = "BuilderMeshRepair";
pub const PRINTING: &str = "Builder3dPrinting";
pub const PROFILE: &str = "BuilderProfile";
pub const PROFILE_MERGE: &str = "BuilderProfileMerge";
pub const PROFILE_END_CAP: &str = "BuilderProfileEndCap";
pub const CUSTOM_RAIL_ADVANCED: &str = "BuilderCustomRailAdvanced";
pub const OBJECT_ON_CURVE: &str = "BuilderObjectOnCrv";
pub const SURFACE_INSET: &str = "BuilderSurfaceInset";
pub const MESH_REDUCER: &str = "BuilderMeshReducer";
pub const MESH: &str = "BuilderMesh";
pub const CURVE_TRANSFORM: &str = "BuilderCurveTransformTool";
pub const CENTER_LINE: &str = "BuilderCenterLine";

pub struct Tools;

impl Interactive for Tools {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        RING_RAIL,
        PROFILE_PLACER,
        OUTSIDE_RAIL,
        CUSTOM_RAIL,
        FOUR_PROFILE,
        METAL_WEIGHTS,
        RESIZE,
        SURFACE_PULLBACK,
        SETTINGS_LIBRARY,
        FINDINGS_LIBRARY,
        SMART_TARGET,
        SMART_TARGET_ADD,
        SMART_TARGET_END,
        SMART_BLEND,
        SMART_MSR,
        JOIN_HISTORY,
        IMAGE_TRACE,
        OBJECT_CHECKER,
        MESH_REPAIR,
        PRINTING,
        PROFILE,
        PROFILE_MERGE,
        PROFILE_END_CAP,
        CUSTOM_RAIL_ADVANCED,
        OBJECT_ON_CURVE,
        SURFACE_INSET,
        MESH_REDUCER,
        MESH,
        CURVE_TRANSFORM,
        CENTER_LINE,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            RING_RAIL => ring_rail(e),
            PROFILE_PLACER => profile_placer(e),
            OUTSIDE_RAIL => outside_rail(e),
            CUSTOM_RAIL | CUSTOM_RAIL_ADVANCED => custom_rail(e),
            FOUR_PROFILE => four_profile(e),
            METAL_WEIGHTS => metal_weights(e),
            RESIZE => resize(e),
            SURFACE_PULLBACK => surface_pullback(e),
            SETTINGS_LIBRARY | FINDINGS_LIBRARY => library(e),
            SMART_TARGET => smart_target(e),
            SMART_TARGET_ADD => smart_target_add(e),
            SMART_TARGET_END => smart_target_end(e),
            SMART_BLEND => smart_blend(e),
            SMART_MSR => smart_msr(e),
            JOIN_HISTORY => join(e),
            IMAGE_TRACE => image_trace(e),
            OBJECT_CHECKER => object_checker(e),
            MESH_REPAIR => mesh_repair(e),
            PRINTING => printing(e),
            PROFILE => profile(e),
            PROFILE_MERGE => profile_merge(e),
            PROFILE_END_CAP => end_cap(e),
            OBJECT_ON_CURVE => object_on_curve(e),
            SURFACE_INSET => surface_inset(e),
            MESH_REDUCER => mesh_reducer(e),
            MESH => to_mesh_command(e),
            CURVE_TRANSFORM => curve_transform(e),
            CENTER_LINE => center_line(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- shared ----------------------------------------------------------------------------------

/// Inner diameter (mm) of a ring size: US/Canada, ISO (inner circumference) or a diameter.
pub fn inner_diameter(system: &str, size: f64) -> f64 {
    match system {
        "diameter" => size,
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

/// The finger circle in the Front view: start at the top, turning toward +X.
pub fn rail_circle(r: f64) -> Geo {
    Geo::Circle {
        center: [0.0; 3],
        x: [0.0, 0.0, 1.0],
        y: [1.0, 0.0, 0.0],
        radius: r,
    }
}

fn rail_point(r: f64, t: f64) -> (Vec3, Vec3) {
    let n = [t.sin(), 0.0, t.cos()];
    (scale(n, r), n)
}

fn to_mesh(m: &SelectedMesh) -> Mesh {
    Mesh {
        vertices: m.vertices.clone(),
        triangles: m.triangles.clone(),
    }
}

fn sel_mesh(m: &Mesh) -> SelectedMesh {
    SelectedMesh {
        vertices: m.vertices.clone(),
        triangles: m.triangles.clone(),
    }
}

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

fn solids(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| {
            s.gem.is_none()
                && s.kind != "curve"
                && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty())
        })
        .collect()
}

fn smooth(points: &[Vec3], closed: bool) -> Option<Geo> {
    if points.len() == 2 && !closed {
        return Some(Geo::Line {
            start: points[0],
            end: points[1],
        });
    }
    interpolate(points, 3, closed)
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

fn replaced(
    e: &Event,
    label: &str,
    ask: &str,
    make: impl FnOnce() -> Result<(Vec<(String, Geo)>, String), String>,
) -> Reply {
    match make() {
        Ok((items, what)) if !items.is_empty() => {
            if matches!(e.input, Input::Enter) {
                let mut r = Reply::finish(Vec::new()).say(format!("{label}: {what}"));
                r.replace = items
                    .into_iter()
                    .map(|(id, geo)| Replacement { id, geo })
                    .collect();
                r
            } else {
                let preview = items
                    .iter()
                    .flat_map(|(_, g)| match g {
                        Geo::Mesh {
                            vertices,
                            triangles,
                            ..
                        } => Mesh {
                            vertices: vertices.clone(),
                            triangles: triangles.clone(),
                        }
                        .wire(),
                        g => vec![g.clone()],
                    })
                    .collect();
                Reply::ask(format!("{label}: {what}. Enter to apply"), json!({}))
                    .want(Want::Objects)
                    .preview(preview)
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

/// A closed profile `(across, up)` of `width` × `height` centered on its base line.
pub fn profile_shape(kind: &str, width: f64, height: f64) -> Vec<[f64; 2]> {
    let (w, h) = (width / 2.0, height);
    let n = 16;
    match kind {
        "d" => {
            let mut p = vec![[w, 0.0]];
            for k in 0..=n {
                let a = PI * k as f64 / n as f64;
                p.push([w * a.cos(), h * a.sin()]);
            }
            p.push([-w, 0.0]);
            p.dedup();
            p.pop();
            p
        }
        "comfort" => (0..2 * n)
            .map(|k| {
                let a = TAU * k as f64 / (2 * n) as f64;
                let (c, s) = (a.cos(), a.sin());
                let up = if s >= 0.0 {
                    h * (0.15 + 0.85 * s)
                } else {
                    h * 0.15 * (1.0 + s)
                };
                [w * c.signum() * c.abs().powf(0.6), up]
            })
            .collect(),
        "knife" => vec![[w, 0.0], [w, h * 0.4], [0.0, h], [-w, h * 0.4], [-w, 0.0]],
        "round" => (0..2 * n)
            .map(|k| {
                let a = TAU * k as f64 / (2 * n) as f64;
                [w * a.cos(), h / 2.0 + h / 2.0 * a.sin()]
            })
            .collect(),
        _ => vec![[w, 0.0], [w, h], [-w, h], [-w, 0.0]],
    }
}

// ---- ring rails and profiles -----------------------------------------------------------------

fn ring_rail(e: &Event) -> Reply {
    let r = ring_radius(e, "rr");
    built(
        e,
        "Ring Rail",
        "choose the size in the builder, Enter",
        || {
            Ok((
                vec![rail_circle(r)],
                format!(
                    "{} mm inside diameter, {} mm around",
                    fmt(2.0 * r),
                    fmt(TAU * r)
                ),
            ))
        },
    )
}

/// The ring rail under the selection (a circle in the Front view), else the size set.
fn rail_radius(e: &Event, p: &str) -> f64 {
    for s in &e.selection {
        if let Some(Geo::Circle { radius, .. }) = &s.exact {
            return *radius;
        }
    }
    ring_radius(e, p)
}

/// Profiles on the rail: closed curves standing across it at `angles` (0 = top), each sized
/// `(width, height)`, their base on the rail, growing outward.
pub fn rail_profiles(r: f64, kind: &str, stations: &[(f64, f64, f64)]) -> Vec<Geo> {
    stations
        .iter()
        .map(|&(t, w, h)| {
            let (c, n) = rail_point(r, t);
            let pts: Vec<Vec3> = profile_shape(kind, w, h)
                .iter()
                .map(|q| add(c, add(scale([0.0, 1.0, 0.0], q[0]), scale(n, q[1]))))
                .collect();
            Geo::Polyline {
                points: pts,
                closed: true,
            }
        })
        .collect()
}

fn profile_placer(e: &Event) -> Reply {
    let kind = e.text("pp_shape", "comfort").to_owned();
    let (tw, th) = (
        e.number("pp_top_width", 4.0),
        e.number("pp_top_height", 2.0),
    );
    let (sw, sh) = (
        e.number("pp_side_width", 3.0),
        e.number("pp_side_height", 1.8),
    );
    let (bw, bh) = (
        e.number("pp_bottom_width", 2.5),
        e.number("pp_bottom_height", 1.5),
    );
    let five = e.number("pp_count", 3.0).round() as usize >= 5;
    built(
        e,
        "Profile Placer",
        "select the ring rail (or use the size), Enter",
        || {
            let r = rail_radius(e, "pp");
            let mut st = vec![
                (0.0, tw, th),
                (PI / 2.0, sw, sh),
                (PI, bw, bh),
                (-PI / 2.0, sw, sh),
            ];
            if five {
                let mid = |a: f64, b: f64| (a + b) / 2.0;
                st.push((PI / 4.0, mid(tw, sw), mid(th, sh)));
                st.push((-PI / 4.0, mid(tw, sw), mid(th, sh)));
            }
            Ok((
                rail_profiles(r, &kind, &st),
                format!("{} profiles on a {} mm rail", st.len(), fmt(2.0 * r)),
            ))
        },
    )
}

/// The outside rail: thickness from the finger `top`, `side` and `bottom`, smooth between.
pub fn outside_rail_points(r: f64, top: f64, side: f64, bottom: f64, n: usize) -> Vec<Vec3> {
    (0..n)
        .map(|k| {
            let t = TAU * k as f64 / n as f64;
            let c = t.cos();
            // Blend top↔side↔bottom by cos: c = 1 top, 0 side, −1 bottom.
            let th = if c >= 0.0 {
                side + (top - side) * c * c
            } else {
                side + (bottom - side) * c * c
            };
            rail_point(r + th, t).0
        })
        .collect()
}

fn outside_rail(e: &Event) -> Reply {
    let (t, s, b) = (
        e.number("orr_top", 3.0),
        e.number("orr_side", 1.8),
        e.number("orr_bottom", 1.5),
    );
    built(
        e,
        "Outside Ring Rail",
        "select the ring rail (or use the size), Enter",
        || {
            let r = rail_radius(e, "orr");
            let g = smooth(&outside_rail_points(r, t, s, b, 64), true).ok_or("rail")?;
            Ok((
                vec![g],
                format!("top {} / side {} / bottom {} mm", fmt(t), fmt(s), fmt(b)),
            ))
        },
    )
}

/// y of a top-view shape curve at x (first crossing), linear between samples.
fn y_at(shape: &[Vec3], x: f64) -> Option<f64> {
    shape.windows(2).find_map(|w| {
        let (a, b) = (w[0], w[1]);
        ((a[0] - x) * (b[0] - x) <= 0.0 && (b[0] - a[0]).abs() > 1e-12)
            .then(|| a[1] + (b[1] - a[1]) * (x - a[0]) / (b[0] - a[0]))
    })
}

fn custom_rail(e: &Event) -> Reply {
    let advanced = e.command == CUSTOM_RAIL_ADVANCED;
    let label = if advanced {
        "Custom Rail Advanced"
    } else {
        "Custom Rail"
    };
    let mirror_y = e.flag("cr_mirror", false);
    built(
        e,
        label,
        "select the ring rail and the shape curve(s) drawn in the Top view, Enter",
        || {
            let cs = curves(e);
            let rail = cs
                .iter()
                .find(|c| {
                    matches!(c.exact, Some(Geo::Circle { .. }))
                        || c.points.iter().all(|p| p[1].abs() < 1e-6)
                })
                .ok_or("select the ring rail")?;
            let shapes: Vec<&&Selected> = cs.iter().filter(|c| c.id != rail.id).collect();
            let top = shapes.first().ok_or("select the shape curve (Top view)")?;
            // Advanced: a second shape (Side view, y against z) blended in by the angle.
            let side = if advanced { shapes.get(1) } else { None };
            let side_pts: Option<Vec<Vec3>> =
                side.map(|s| s.points.iter().map(|p| [p[2], p[1], 0.0]).collect());
            let mut pts = Vec::new();
            for &p in &rail.points {
                let Some(yt) = y_at(&top.points, p[0]) else {
                    continue;
                };
                let y = match &side_pts {
                    Some(sp) => {
                        let ys = y_at(sp, p[2]).unwrap_or(yt);
                        let r = norm(p).max(1e-9);
                        let w = (p[2] / r).abs();
                        yt * w + ys * (1.0 - w)
                    }
                    None => yt,
                };
                pts.push([p[0], y, p[2]]);
            }
            if pts.len() < 4 {
                return Err("the shape curve must span the rail across the Top view".into());
            }
            let mut out: Vec<Geo> = smooth(&pts, rail.closed && pts.len() == rail.points.len())
                .into_iter()
                .collect();
            if mirror_y {
                let m: Vec<Vec3> = pts.iter().map(|p| [p[0], -p[1], p[2]]).collect();
                out.extend(smooth(&m, rail.closed && pts.len() == rail.points.len()));
            }
            let n = out.len();
            Ok((out, format!("{n} rails")))
        },
    )
}

fn four_profile(e: &Event) -> Reply {
    let flow = Flow::new("Four Rail Profile")
        .point("Start of the profile line")
        .point("End of the profile line");
    let make = |c: &Collected, end: Option<Vec3>| -> Result<Geo, String> {
        let cs = curves(e);
        if cs.len() != 4 {
            return Err("select four curves".into());
        }
        let (Some(a), Some(b)) = (c.p(0), c.p(1).or(end)) else {
            return Err("pick the line".into());
        };
        let n = unit(cross(sub(b, a), e.plane.normal()));
        let mut hits = Vec::new();
        for s in &cs {
            let p = &s.points;
            let m = if s.closed { p.len() } else { p.len() - 1 };
            let hit = (0..m).find_map(|i| {
                let (x, y) = (p[i], p[(i + 1) % p.len()]);
                let (dx, dy) = (dot(sub(x, a), n), dot(sub(y, a), n));
                ((dx > 0.0) != (dy > 0.0)).then(|| add(x, scale(sub(y, x), dx / (dx - dy))))
            });
            hits.push(hit.ok_or("a curve does not cross the line's plane")?);
        }
        let c0 = scale(hits.iter().fold([0.0; 3], |s, &p| add(s, p)), 0.25);
        let f = Frame::new(c0, n, sub(hits[0], c0));
        hits.sort_by(|p, q| {
            let (lp, lq) = (f.local(*p), f.local(*q));
            lp[1].atan2(lp[0]).total_cmp(&lq[1].atan2(lq[0]))
        });
        smooth(&hits, true).ok_or_else(|| "profile".into())
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            let g = make(c, None)?;
            let mut r =
                Reply::finish(vec![g]).say("Four Rail Profile: profile through the four rails");
            if e.flag("frp_delete", false) {
                r.delete = curves(e).iter().map(|s| s.id.clone()).collect();
            }
            Ok(r)
        },
    )
}

fn profile(e: &Event) -> Reply {
    let kind = e.text("prof_shape", "comfort").to_owned();
    let w = e.number("prof_width", 4.0);
    let h = e.number("prof_height", 2.0);
    let flow = Flow::new("Profile")
        .no_selection()
        .point("Where to place the profile (Top view)");
    let make = |p: Vec3| -> Geo {
        let l = e.plane.local(p);
        Geo::Polyline {
            points: profile_shape(&kind, w, h)
                .iter()
                .map(|q| e.plane.at(l[0] + q[0], l[1] + q[1], l[2]))
                .collect(),
            closed: true,
        }
    };
    flow.step(
        e,
        &|_, h| Show {
            geo: h.map(make).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            Ok(Reply::finish(vec![make(c.p(0).unwrap())]).say(format!(
                "Profile: {kind} {} × {} mm",
                fmt(w),
                fmt(h)
            )))
        },
    )
}

/// Union / intersection / difference of closed planar outlines (through prisms).
pub fn outline_boolean(outlines: &[Vec<Vec3>], normal: Vec3, op: Op) -> Vec<(Vec<Vec3>, bool)> {
    let prisms: Vec<Mesh> = outlines
        .iter()
        .filter_map(|o| {
            let pts: Vec<Vec3> = o.iter().map(|&p| sub(p, normal)).collect();
            shapes::prism(&pts, &[], scale(normal, 2.0))
        })
        .collect();
    let Some(r) = csg::fold(&prisms, op) else {
        return Vec::new();
    };
    section::section(&sel_mesh(&r), outlines[0][0], normal)
}

fn profile_merge(e: &Event) -> Reply {
    built(
        e,
        "Profile Merge",
        "select two overlapping closed profiles, Enter",
        || {
            let cs: Vec<&Selected> = curves(e).into_iter().filter(|c| c.closed).collect();
            if cs.len() < 2 {
                return Err("select two closed profiles".into());
            }
            let n = shapes::loop_plane(&cs[0].points).map_or(e.plane.normal(), |p| p.1);
            let out: Vec<Geo> = outline_boolean(
                &cs.iter().map(|c| c.points.clone()).collect::<Vec<_>>(),
                n,
                Op::Union,
            )
            .into_iter()
            .map(|(p, closed)| Geo::Polyline { points: p, closed })
            .collect();
            Ok((out, "merged outline".into()))
        },
    )
}

fn end_cap(e: &Event) -> Reply {
    let amount = e.number("pec_amount", 1.5);
    let flip = e.flag("pec_flip", false);
    built(
        e,
        "Profile End Cap",
        "select the closed end profile, Enter",
        || {
            let c = curves(e)
                .into_iter()
                .find(|c| c.closed)
                .ok_or("select a closed profile")?;
            let (center, mut n) =
                shapes::loop_plane(&c.points).ok_or("the profile is not planar")?;
            if flip {
                n = scale(n, -1.0);
            }
            // A dome from the profile out along its normal: rings shrinking to the tip.
            let k = 10;
            let mut rings: Vec<Vec<Vec3>> = (0..k)
                .map(|i| {
                    let a = PI / 2.0 * i as f64 / k as f64;
                    let s = a.cos();
                    c.points
                        .iter()
                        .map(|&p| {
                            add(
                                add(center, scale(sub(p, center), s)),
                                scale(n, amount * a.sin()),
                            )
                        })
                        .collect()
                })
                .collect();
            rings.reverse();
            let m = mesh::ring_stack(&rings, Some(add(center, scale(n, amount))), Some(center));
            let m = if m.volume() < 0.0 {
                Mesh {
                    triangles: m.triangles.iter().map(|t| [t[0], t[2], t[1]]).collect(),
                    ..m
                }
            } else {
                m
            };
            Ok((
                vec![m.geo("End cap", None, None)],
                format!("cap {} mm", fmt(amount)),
            ))
        },
    )
}

// ---- smart targets ---------------------------------------------------------------------------

/// A smart target: a blend point with a handle (a line centered on the point).
pub fn target_line(p: Vec3, dir: Vec3, handle: f64) -> Geo {
    let d = scale(unit(dir), handle / 2.0);
    Geo::Line {
        start: sub(p, d),
        end: add(p, d),
    }
}

fn smart_target(e: &Event) -> Reply {
    let handle = e.number("st_handle", 2.0);
    let flow = Flow::new("Smart Target")
        .no_selection()
        .point("Blend point")
        .point("Direction of the handle");
    flow.step(
        e,
        &|c, h| Show {
            geo: match (c.p(0), h) {
                (Some(p), Some(q)) if dist(p, q) > 1e-9 => vec![target_line(p, sub(q, p), handle)],
                _ => Vec::new(),
            },
            ..Default::default()
        },
        &|c| {
            let (p, q) = (c.p(0).unwrap(), c.p(1).unwrap());
            if dist(p, q) < 1e-9 {
                return Err("pick a direction away from the point".into());
            }
            Ok(Reply::finish(vec![
                Geo::Point { point: p },
                target_line(p, sub(q, p), handle),
            ])
            .say("Smart Target: blend point and handle"))
        },
    )
}

/// The blend point and direction of a selected smart target (a line or polyline handle).
fn target_of(s: &Selected) -> Option<(Vec3, Vec3)> {
    let p = &s.points;
    if s.kind != "curve" || p.len() < 2 {
        return None;
    }
    let (a, b) = (p[0], *p.last().unwrap());
    (dist(a, b) > 1e-9).then(|| (scale(add(a, b), 0.5), unit(sub(b, a))))
}

fn smart_target_add(e: &Event) -> Reply {
    let handle = e.number("st_handle", 2.0);
    let flow = Flow::new("Smart Target Add").point("Direction of the new handle");
    let at = || curves(e).first().and_then(|s| target_of(s)).map(|t| t.0);
    flow.step(
        e,
        &|_, h| Show {
            geo: match (at(), h) {
                (Some(p), Some(q)) if dist(p, q) > 1e-9 => vec![target_line(p, sub(q, p), handle)],
                _ => Vec::new(),
            },
            ..Default::default()
        },
        &|c| {
            let p = at().ok_or("select a smart target")?;
            let q = c.p(0).unwrap();
            if dist(p, q) < 1e-9 {
                return Err("pick a direction away from the point".into());
            }
            Ok(Reply::finish(vec![target_line(p, sub(q, p), handle)])
                .say("Smart Target Add: handle added"))
        },
    )
}

fn smart_target_end(e: &Event) -> Reply {
    let handle = e.number("st_handle", 2.0);
    let flow = Flow::new("Smart Target on Curve End").point("Pick near the curve end");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let q = c.p(0).unwrap();
        let s = curves(e).into_iter().next().ok_or("select a curve")?;
        let p = &s.points;
        let (end, prev) = if dist(q, p[0]) <= dist(q, *p.last().unwrap()) {
            (p[0], p[1])
        } else {
            (*p.last().unwrap(), p[p.len() - 2])
        };
        let dir = unit(sub(end, prev));
        // The handle's back half lies on the curve, the front half continues it.
        Ok(Reply::finish(vec![
            Geo::Point { point: end },
            target_line(end, dir, handle),
        ])
        .say("Smart Target on Curve End: target at the end"))
    })
}

fn smart_blend(e: &Event) -> Reply {
    let bulge = e.number("sb_bulge", 1.0);
    built(e, "Smart Blend", "select two smart targets, Enter", || {
        let ts: Vec<(Vec3, Vec3)> = curves(e).iter().filter_map(|s| target_of(s)).collect();
        if ts.len() != 2 {
            return Err("select two smart targets".into());
        }
        let ((p0, mut d0), (p1, mut d1)) = (ts[0], ts[1]);
        // Leave the first toward the second; arrive along the second's handle.
        if dot(d0, sub(p1, p0)) < 0.0 {
            d0 = scale(d0, -1.0);
        }
        if dot(d1, sub(p1, p0)) < 0.0 {
            d1 = scale(d1, -1.0);
        }
        Ok((
            vec![nurbs::blend(p0, d0, p1, d1, [bulge, bulge], false)],
            "blend between the targets".into(),
        ))
    })
}

fn smart_msr(e: &Event) -> Reply {
    let flow = Flow::new("Smart MSR")
        .number("Rotate the handles (degrees)", 0.0)
        .number("Scale the handles (%)", 100.0)
        .point("Move from")
        .point("Move to");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (ang, sc) = (c.n(0).unwrap().to_radians(), c.n(1).unwrap() / 100.0);
        let mv = sub(c.p(1).unwrap(), c.p(0).unwrap());
        let n = e.plane.normal();
        let items: Vec<ObjectMatrix> = curves(e)
            .iter()
            .filter_map(|s| target_of(s).map(|t| (s, t)))
            .map(|(s, (p, d))| {
                let from = Frame::new(p, n, d);
                let to = Frame::new(add(p, mv), n, d).rotated(ang);
                ObjectMatrix {
                    id: s.id.clone(),
                    matrix: frame_matrix(&from, &to, sc),
                }
            })
            .collect();
        if items.is_empty() {
            return Err("select smart targets".into());
        }
        let k = items.len();
        Ok(Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: false,
            per_object: items,
        })
        .say(format!("Smart MSR: {k} targets")))
    })
}

/// Join curves end to end into one (nearest ends), reversing pieces as needed.
pub fn join_curves(pieces: &[Vec<Vec3>], tol: f64) -> Vec<(Vec<Vec3>, bool)> {
    let mut left: Vec<Vec<Vec3>> = pieces.to_vec();
    let mut out = Vec::new();
    while let Some(mut chain) = left.pop() {
        loop {
            let tail = *chain.last().unwrap();
            let head = chain[0];
            let mut found = None;
            for (i, p) in left.iter().enumerate() {
                let (a, b) = (p[0], *p.last().unwrap());
                if dist(tail, a) <= tol {
                    found = Some((i, false, false));
                    break;
                }
                if dist(tail, b) <= tol {
                    found = Some((i, true, false));
                    break;
                }
                if dist(head, b) <= tol {
                    found = Some((i, false, true));
                    break;
                }
                if dist(head, a) <= tol {
                    found = Some((i, true, true));
                    break;
                }
            }
            let Some((i, rev, front)) = found else { break };
            let mut p = left.remove(i);
            if rev {
                p.reverse();
            }
            if front {
                p.pop();
                p.extend(chain);
                chain = p;
            } else {
                chain.extend(p.into_iter().skip(1));
            }
        }
        let closed = chain.len() > 2 && dist(chain[0], *chain.last().unwrap()) <= tol;
        if closed {
            chain.pop();
        }
        out.push((chain, closed));
    }
    out
}

fn join(e: &Event) -> Reply {
    let tol = e.number("jh_tolerance", 0.01);
    let flow = Flow::new("Join");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let cs = curves(e);
        if cs.len() < 2 {
            return Err("select two or more curves that meet".into());
        }
        let joined = join_curves(
            &cs.iter().map(|c| c.points.clone()).collect::<Vec<_>>(),
            tol,
        );
        let out: Vec<Geo> = joined
            .into_iter()
            .map(|(p, closed)| Geo::Polyline { points: p, closed })
            .collect();
        let n = out.len();
        let mut r = Reply::finish(out).say(format!("Join: {} curves into {n}", cs.len()));
        r.delete = cs.iter().map(|c| c.id.clone()).collect();
        Ok(r)
    })
}

// ---- metal weights, resizer, pullback --------------------------------------------------------

/// Metal densities (g/cm³).
pub const METALS: &[(&str, f64)] = &[
    ("24k gold", 19.32),
    ("22k gold", 17.8),
    ("18k yellow gold", 15.5),
    ("18k white gold", 15.8),
    ("14k yellow gold", 13.1),
    ("14k white gold", 13.4),
    ("10k gold", 11.6),
    ("platinum 950", 20.7),
    ("palladium 950", 12.0),
    ("sterling silver", 10.36),
    ("brass", 8.5),
    ("wax", 1.0),
];

fn metal_weights(e: &Event) -> Reply {
    let label = "Metal Weights";
    let objs = solids(e);
    let (gems, _) = selected_gems(e);
    let vol: f64 = objs
        .iter()
        .map(|s| s.mesh.as_ref().unwrap().volume().abs())
        .sum();
    let open = objs
        .iter()
        .filter(|s| s.mesh.as_ref().unwrap().open_and_nonmanifold_edges().0 > 0)
        .count();
    if objs.is_empty() && gems.is_empty() {
        let r = Reply::ask(
            format!("{label}: select the metal (and stones), Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select solids"))
        } else {
            r
        };
    }
    let mut lines = vec![format!("{label}: {} objects, {} mm³", objs.len(), fmt(vol))];
    for (m, d) in METALS {
        let g = vol / 1000.0 * d;
        lines.push(format!("  {m}: {} g ({} dwt)", fmt(g), fmt(g / 1.55517)));
    }
    if !gems.is_empty() {
        let ct: f64 = gems.iter().map(|g| g.gem.carat(&g.material)).sum();
        lines.push(format!(
            "  {} stones: {} ct ({} g)",
            gems.len(),
            fmt(ct),
            fmt(ct * 0.2)
        ));
    }
    if open > 0 {
        lines.push(format!(
            "  warning: {open} objects are open: their volume is not reliable"
        ));
    }
    if matches!(e.input, Input::Enter) {
        let mut r = Reply::finish(Vec::new());
        r.messages = lines;
        return r;
    }
    Reply::ask(
        format!(
            "{label}: {} mm³ = {} g in 18k yellow gold. Enter for the list",
            fmt(vol),
            fmt(vol / 1000.0 * 15.5)
        ),
        json!({}),
    )
    .want(Want::Objects)
}

/// Resize a ring about the world Y axis: every point moves out by the change of the finger
/// radius, so the shank keeps its thickness.
pub fn resized_ring(m: &SelectedMesh, from_r: f64, to_r: f64) -> SelectedMesh {
    let dr = to_r - from_r;
    let mut out = m.clone();
    for v in &mut out.vertices {
        let rho = (v[0] * v[0] + v[2] * v[2]).sqrt();
        if rho > 1e-9 {
            let k = (rho + dr) / rho;
            v[0] *= k;
            v[2] *= k;
        }
    }
    out
}

fn resize(e: &Event) -> Reply {
    let from = inner_diameter(
        e.text("rs_size_system", "us"),
        e.number("rs_ring_size", 7.0),
    ) / 2.0;
    let sizes: Vec<f64> = e
        .text("rs_sizes", "6, 8")
        .split([',', ';', ' '])
        .filter_map(|t| t.trim().parse().ok())
        .collect();
    let gap = e.number("rs_spacing", 30.0);
    let system = e.text("rs_size_system", "us").to_owned();
    built(
        e,
        "Ring Resizer",
        "select the ring (no stones), Enter",
        || {
            let objs = solids(e);
            if objs.is_empty() || sizes.is_empty() {
                return Err("select the ring and list the sizes".into());
            }
            let mut out = Vec::new();
            for (k, s) in sizes.iter().enumerate() {
                let to = inner_diameter(&system, *s) / 2.0;
                for o in &objs {
                    let mut m = resized_ring(o.mesh.as_ref().unwrap(), from, to);
                    for v in &mut m.vertices {
                        v[0] += gap * (k + 1) as f64;
                    }
                    out.push(Geo::Mesh {
                        name: format!("Ring size {}", fmt(*s)),
                        vertices: m.vertices,
                        triangles: m.triangles,
                        meta: None,
                        layer: None,
                    });
                }
            }
            Ok((
                out,
                format!(
                    "{} sizes: {}",
                    sizes.len(),
                    sizes.iter().map(|s| fmt(*s)).collect::<Vec<_>>().join(", ")
                ),
            ))
        },
    )
}

#[derive(Default, Serialize, Deserialize)]
struct Group {
    items: Vec<Selected>,
}

fn surface_pullback(e: &Event) -> Reply {
    let label = "Surface Pullback";
    let mut st: Group = load(e);
    if st.items.is_empty() {
        if matches!(e.input, Input::Enter) && !e.selection.is_empty() {
            st.items = e.selection.clone();
            return Reply::ask(
                format!(
                    "{label}: {} objects. Now select the target surface, Enter",
                    st.items.len()
                ),
                save(&st),
            )
            .want(Want::Objects);
        }
        return Reply::ask(
            format!("{label}: select the flat design (curves, meshes) lying on the CPlane, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    }
    let ids: Vec<&str> = st.items.iter().map(|s| s.id.as_str()).collect();
    let Some(target) = e
        .selection
        .iter()
        .find(|s| !ids.contains(&s.id.as_str()) && s.mesh.is_some())
    else {
        return Reply::ask(
            format!("{label}: select the target surface, Enter"),
            save(&st),
        )
        .want(Want::Objects);
    };
    let n = e.plane.normal();
    let t = Target::new(target.mesh.as_ref().unwrap(), n);
    // Each point drops onto the surface and keeps its height above the CPlane along the
    // surface normal there.
    let pull = |p: Vec3| -> Vec3 {
        let h = e.plane.local(p)[2];
        t.as_ref()
            .and_then(|t| t.drop(add(p, scale(n, 1e6))))
            .map_or(p, |(q, nn)| add(q, scale(nn, h)))
    };
    let mut out = Vec::new();
    for s in &st.items {
        if let Some(m) = &s.mesh {
            let v: Vec<Vec3> = m.vertices.iter().map(|&p| pull(p)).collect();
            out.push(Geo::Mesh {
                name: format!("{} on surface", s.name),
                vertices: v,
                triangles: m.triangles.clone(),
                meta: None,
                layer: None,
            });
        } else if s.kind == "curve" {
            let p: Vec<Vec3> = s.points.iter().map(|&p| pull(p)).collect();
            out.extend(smooth(&p, s.closed));
        }
    }
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!("{label}: {} objects onto the surface, Enter", out.len()),
            save(&st),
        )
        .want(Want::Objects)
        .preview(
            out.iter()
                .filter(|g| !matches!(g, Geo::Mesh { .. }))
                .cloned()
                .collect(),
        );
    }
    let k = out.len();
    Reply::finish(out).say(format!("{label}: {k} objects"))
}

// ---- findings libraries ----------------------------------------------------------------------

/// Our own parametric findings and settings (no vendor parts are shipped).
pub fn finding(kind: &str, size: f64) -> Mesh {
    let s = size;
    match kind {
        // Earring post with a ball at its base.
        "post" => {
            let mut m = mesh::cylinder(0.4 * s / 10.0 + 0.35, 0.0, s, 16);
            m.append(&mesh::sphere(0.8 + s / 20.0, 16));
            m
        }
        // A pendant bail: a U strip.
        "bail" => {
            let path: Vec<Vec3> = (0..=24)
                .map(|k| {
                    let a = PI * k as f64 / 24.0;
                    [s / 2.0 * a.cos(), 0.0, s / 2.0 * a.sin() + s / 2.0]
                })
                .collect();
            mesh::sweep(
                &path,
                false,
                [0.0, 1.0, 0.0],
                &[[0.6, -0.4], [0.6, 0.4], [-0.6, 0.4], [-0.6, -0.4]],
            )
            .unwrap_or_default()
        }
        "jump_ring" => mesh::torus(s / 2.0, 0.4, 32, 12),
        // A friction back (butterfly): a disc with a tube.
        "clutch" => {
            let mut m = mesh::cylinder(s / 2.0, 0.0, 0.4, 24);
            m.append(&mesh::cylinder(0.6, 0.4, 2.0, 12));
            m
        }
        // Spring ring clasp: a torus with a pull.
        "clasp" => {
            let mut m = mesh::torus(s / 2.0, s / 7.0, 32, 12);
            m.append(
                &mesh::cylinder(s / 8.0, s / 2.0, s / 2.0 + s / 3.0, 12).placed(&Frame::new(
                    [0.0; 3],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                )),
            );
            m
        }
        // Settings library: peg head (4 prongs on a ring), cup, basket.
        "peg_head" | "basket" => {
            let mut m = mesh::tube(
                &[
                    mesh::circle(s / 2.0 + 0.4, -s * 0.3, 32),
                    mesh::circle(s / 2.0 + 0.4, -s * 0.45, 32),
                ],
                &[
                    mesh::circle(s / 2.0, -s * 0.3, 32),
                    mesh::circle(s / 2.0, -s * 0.45, 32),
                ],
            );
            let n = if kind == "basket" { 6 } else { 4 };
            for k in 0..n {
                let a = TAU * (k as f64 + 0.5) / n as f64;
                let base = [(s / 2.0) * a.cos(), (s / 2.0) * a.sin(), 0.0];
                let mut p = mesh::domed_cylinder(0.08 * s + 0.15, -s * 0.9, 0.25 * s, 12);
                p = p.placed(&Frame::new(base, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]));
                m.append(&p);
            }
            if kind == "peg_head" {
                m.append(&mesh::cylinder(0.12 * s + 0.2, -s * 1.8, -s * 0.9, 12));
            }
            m
        }
        "cup" => mesh::tube(
            &[
                mesh::circle(s / 2.0 + 0.4, 0.2, 32),
                mesh::circle(s / 2.0 * 0.6, -s * 0.5, 32),
            ],
            &[
                mesh::circle(s / 2.0, 0.2, 32),
                mesh::circle(s / 2.0 * 0.6 - 0.3, -s * 0.5 + 0.3, 32),
            ],
        ),
        _ => mesh::sphere(s / 2.0, 16),
    }
}

fn library(e: &Event) -> Reply {
    let settings = e.command == SETTINGS_LIBRARY;
    let (p, label) = if settings {
        ("lib_setting", "Settings Library")
    } else {
        ("lib_finding", "Findings Library")
    };
    let kind = e
        .text(p, if settings { "peg_head" } else { "post" })
        .to_owned();
    let size = e.number(&format!("{p}_size"), if settings { 5.0 } else { 10.0 });
    let flow = Flow::new(label).no_selection().point("Where to place it");
    let make = |at: Vec3| finding(&kind, size).placed(&Frame::new(at, e.plane.normal(), e.plane.x));
    flow.step(
        e,
        &|_, h| Show {
            geo: h.map(|p| make(p).wire()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(Reply::finish(vec![make(c.p(0).unwrap()).geo(
                &format!("{label}: {kind}"),
                None,
                None,
            )])
            .say(format!("{label}: {kind} {} mm", fmt(size))))
        },
    )
}

// ---- image trace -----------------------------------------------------------------------------

/// A greyscale image (0 = black … 1 = white), row 0 at the top.
pub struct Grey {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f64>,
}

/// Read PNG, BMP (24/32-bit) or PGM/PPM (binary) files.
pub fn read_image(path: &std::path::Path) -> Result<Grey, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        let mut dec = png::Decoder::new(std::io::Cursor::new(&bytes));
        dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut r = dec.read_info().map_err(|e| e.to_string())?;
        let mut buf = vec![0; r.output_buffer_size().ok_or("png too large")?];
        let info = r.next_frame(&mut buf).map_err(|e| e.to_string())?;
        let ch = info.color_type.samples();
        let (w, h) = (info.width as usize, info.height as usize);
        let px = (0..w * h)
            .map(|i| {
                let p = &buf[i * ch..i * ch + ch];
                let (g, a) = match ch {
                    1 => (p[0] as f64, 255.0),
                    2 => (p[0] as f64, p[1] as f64),
                    3 => (
                        0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64,
                        255.0,
                    ),
                    _ => (
                        0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64,
                        p[3] as f64,
                    ),
                };
                // Transparent pixels count as white paper.
                (g * a / 255.0 + 255.0 * (1.0 - a / 255.0)) / 255.0
            })
            .collect();
        return Ok(Grey { w, h, px });
    }
    if bytes.starts_with(b"BM") && bytes.len() > 54 {
        let u32_at =
            |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        let off = u32_at(10) as usize;
        let w = u32_at(18) as i32;
        let hh = u32_at(22) as i32;
        let bpp = u16::from_le_bytes([bytes[28], bytes[29]]) as usize;
        if bpp != 24 && bpp != 32 {
            return Err("only 24- and 32-bit BMP files".into());
        }
        let (w, h, bottom_up) = (
            w.unsigned_abs() as usize,
            hh.unsigned_abs() as usize,
            hh > 0,
        );
        let row = (w * bpp / 8).div_ceil(4) * 4;
        let mut px = vec![1.0; w * h];
        for y in 0..h {
            let src = if bottom_up { h - 1 - y } else { y };
            for x in 0..w {
                let o = off + src * row + x * bpp / 8;
                if o + 2 < bytes.len() {
                    px[y * w + x] = (0.114 * bytes[o] as f64
                        + 0.587 * bytes[o + 1] as f64
                        + 0.299 * bytes[o + 2] as f64)
                        / 255.0;
                }
            }
        }
        return Ok(Grey { w, h, px });
    }
    if bytes.starts_with(b"P5") || bytes.starts_with(b"P6") {
        let text: String = bytes.iter().take(64).map(|&b| b as char).collect();
        let mut it = text.split_whitespace();
        let magic = it.next().unwrap_or("");
        let w: usize = it.next().and_then(|t| t.parse().ok()).ok_or("pnm")?;
        let h: usize = it.next().and_then(|t| t.parse().ok()).ok_or("pnm")?;
        let maxv: f64 = it.next().and_then(|t| t.parse().ok()).ok_or("pnm")?;
        let ch = if magic == "P6" { 3 } else { 1 };
        let data = &bytes[bytes.len() - w * h * ch..];
        let px = (0..w * h)
            .map(|i| {
                if ch == 1 {
                    data[i] as f64 / maxv
                } else {
                    (0.299 * data[3 * i] as f64
                        + 0.587 * data[3 * i + 1] as f64
                        + 0.114 * data[3 * i + 2] as f64)
                        / maxv
                }
            })
            .collect();
        return Ok(Grey { w, h, px });
    }
    Err("use a PNG, BMP or PGM/PPM image".into())
}

/// Outlines of the dark areas (below `threshold`) by marching squares, in pixel units with
/// y up (row 0 at the top becomes the highest y).
pub fn trace(img: &Grey, threshold: f64) -> Vec<(Vec<Vec3>, bool)> {
    let at = |x: isize, y: isize| -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < img.w
            && (y as usize) < img.h
            && img.px[y as usize * img.w + x as usize] < threshold
    };
    let mut segs = Vec::new();
    let p = |x: f64, y: f64| [x, img.h as f64 - y, 0.0];
    for y in -1..img.h as isize {
        for x in -1..img.w as isize {
            let c = [at(x, y), at(x + 1, y), at(x + 1, y + 1), at(x, y + 1)];
            let idx = c
                .iter()
                .enumerate()
                .fold(0, |a, (i, &b)| a | ((b as usize) << i));
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let (t, r, b, l) = (
                p(fx + 0.5, fy),
                p(fx + 1.0, fy + 0.5),
                p(fx + 0.5, fy + 1.0),
                p(fx, fy + 0.5),
            );
            let edges: &[(Vec3, Vec3)] = match idx {
                1 | 14 => &[(l, t)],
                2 | 13 => &[(t, r)],
                3 | 12 => &[(l, r)],
                4 | 11 => &[(r, b)],
                5 => &[(l, t), (r, b)],
                6 | 9 => &[(t, b)],
                7 | 8 => &[(l, b)],
                10 => &[(t, r), (l, b)],
                _ => &[],
            };
            for &(a, bb) in edges {
                segs.push([a, bb]);
            }
        }
    }
    section::chain(&segs, 1e-6)
}

fn image_trace(e: &Event) -> Reply {
    let file = e.text("it_file", "").to_owned();
    let width = e.number("it_width", 20.0);
    let threshold = e.number("it_threshold", 50.0) / 100.0;
    let smooth_on = e.flag("it_smooth", true);
    let flow = Flow::new("Image Trace")
        .no_selection()
        .point("Lower left corner of the traced image");
    let make = |at: Vec3| -> Result<Vec<Geo>, String> {
        if file.trim().is_empty() {
            return Err("choose the image file in the builder (File=…)".into());
        }
        let img = read_image(std::path::Path::new(&file))?;
        let k = width / img.w.max(1) as f64;
        let l = e.plane.local(at);
        let mut out = Vec::new();
        for (pts, closed) in trace(&img, threshold) {
            if pts.len() < 6 {
                continue;
            }
            let w: Vec<Vec3> = pts
                .iter()
                .map(|q| e.plane.at(l[0] + q[0] * k, l[1] + q[1] * k, l[2]))
                .collect();
            let g = if smooth_on {
                let step = (w.len() / 64).max(1);
                let thin: Vec<Vec3> = w.iter().step_by(step).copied().collect();
                smooth(&thin, closed).unwrap_or(Geo::Polyline { points: w, closed })
            } else {
                Geo::Polyline { points: w, closed }
            };
            out.push(g);
        }
        if out.is_empty() {
            return Err("nothing darker than the threshold".into());
        }
        Ok(out)
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let out = make(c.p(0).unwrap())?;
        let n = out.len();
        Ok(Reply::finish(out).say(format!("Image Trace: {n} curves, {} mm wide", fmt(width))))
    })
}

// ---- checking and repairing meshes -----------------------------------------------------------

fn object_checker(e: &Event) -> Reply {
    let label = "Object Checker";
    let objs = solids(e);
    if objs.is_empty() {
        let r =
            Reply::ask(format!("{label}: select objects, Enter"), json!({})).want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select solids or meshes"))
        } else {
            r
        };
    }
    let mut lines = Vec::new();
    let mut naked = Vec::new();
    let mut bad = 0;
    for s in &objs {
        let m = s.mesh.as_ref().unwrap();
        let (open, nm) = m.open_and_nonmanifold_edges();
        let degenerate = m
            .triangles
            .iter()
            .filter(|t| {
                let [a, b, c] = [
                    m.vertices[t[0] as usize],
                    m.vertices[t[1] as usize],
                    m.vertices[t[2] as usize],
                ];
                norm(cross(sub(b, a), sub(c, a))) < 1e-14
            })
            .count();
        let v = m.volume();
        let ok = open == 0 && nm == 0 && degenerate == 0 && v > 0.0;
        if !ok {
            bad += 1;
        }
        lines.push(format!(
            "{}: {}{} open edges, {} non-manifold, {} degenerate triangles, volume {} mm³{}",
            s.name,
            if ok { "OK — " } else { "PROBLEM — " },
            open,
            nm,
            degenerate,
            fmt(v),
            if v < 0.0 {
                " (normals point inward)"
            } else {
                ""
            }
        ));
        naked.extend(
            section::feature_edges(m, 180.0, true)
                .into_iter()
                .map(|s| Geo::Line {
                    start: s[0],
                    end: s[1],
                }),
        );
    }
    if matches!(e.input, Input::Enter) {
        let mut r = Reply::finish(Vec::new());
        r.messages = lines;
        r.messages.push(format!(
            "{label}: {} of {} objects need repair",
            bad,
            objs.len()
        ));
        return r;
    }
    Reply::ask(
        format!(
            "{label}: {bad} of {} objects have problems (naked edges shown). Enter for the report",
            objs.len()
        ),
        json!({}),
    )
    .want(Want::Objects)
    .preview(naked)
}

/// Weld, drop degenerate triangles, fill holes and turn the normals outward.
pub fn repair(m: &SelectedMesh, tol: f64) -> Mesh {
    let welded = shapes::weld(&to_mesh(m), tol);
    let tris: Vec<[u32; 3]> = welded
        .triangles
        .iter()
        .copied()
        .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
        .filter(|t| {
            let [a, b, c] = [
                welded.vertices[t[0] as usize],
                welded.vertices[t[1] as usize],
                welded.vertices[t[2] as usize],
            ];
            norm(cross(sub(b, a), sub(c, a))) > 1e-16
        })
        .collect();
    let m = Mesh {
        vertices: welded.vertices,
        triangles: tris,
    };
    let (mut m, _) = shapes::cap_holes(&m);
    if m.volume() < 0.0 {
        for t in &mut m.triangles {
            t.swap(1, 2);
        }
    }
    m
}

fn mesh_repair(e: &Event) -> Reply {
    let tol = e.number("mr_tolerance", 0.001);
    replaced(e, "Mesh Repair", "select meshes to repair, Enter", || {
        let objs = solids(e);
        let items: Vec<(String, Geo)> = objs
            .iter()
            .map(|s| {
                (
                    s.id.clone(),
                    repair(s.mesh.as_ref().unwrap(), tol).geo(&s.name, None, None),
                )
            })
            .collect();
        let n = items.len();
        Ok((items, format!("{n} meshes welded, cleaned and closed")))
    })
}

/// Reduce a mesh by clustering vertices on a grid of `cell` mm.
pub fn reduce(m: &SelectedMesh, cell: f64) -> Mesh {
    let mut map = std::collections::HashMap::<[i64; 3], (u32, Vec3, f64)>::new();
    let mut ids = Vec::with_capacity(m.vertices.len());
    let mut next = 0u32;
    for v in &m.vertices {
        let k = [
            (v[0] / cell).floor() as i64,
            (v[1] / cell).floor() as i64,
            (v[2] / cell).floor() as i64,
        ];
        let e = map.entry(k).or_insert_with(|| {
            next += 1;
            (next - 1, [0.0; 3], 0.0)
        });
        e.1 = add(e.1, *v);
        e.2 += 1.0;
        ids.push(e.0);
    }
    let mut verts = vec![[0.0; 3]; next as usize];
    for (_, (i, s, n)) in map {
        verts[i as usize] = scale(s, 1.0 / n);
    }
    let mut seen = std::collections::HashSet::new();
    let tris = m
        .triangles
        .iter()
        .map(|t| [ids[t[0] as usize], ids[t[1] as usize], ids[t[2] as usize]])
        .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
        .filter(|t| {
            let mut k = *t;
            k.sort();
            seen.insert(k)
        })
        .collect();
    Mesh {
        vertices: verts,
        triangles: tris,
    }
}

fn mesh_reducer(e: &Event) -> Reply {
    let target = e.number("mred_percent", 50.0) / 100.0;
    replaced(e, "Mesh Reducer", "select meshes, Enter", || {
        let objs = solids(e);
        let mut what = Vec::new();
        let items: Vec<(String, Geo)> = objs
            .iter()
            .map(|s| {
                let m = s.mesh.as_ref().unwrap();
                let want = ((m.triangles.len() as f64) * target).max(8.0);
                let b = to_mesh(m).bbox();
                let diag = dist(b[0], b[1]).max(1e-6);
                // Search the cell size that gives about the wanted triangle count.
                let (mut lo, mut hi) = (diag * 1e-5, diag / 4.0);
                let mut best = to_mesh(m);
                for _ in 0..30 {
                    let mid = (lo * hi).sqrt();
                    let r = reduce(m, mid);
                    if (r.triangles.len() as f64) > want {
                        lo = mid
                    } else {
                        hi = mid;
                        best = r;
                    }
                }
                what.push(format!("{} → {}", m.triangles.len(), best.triangles.len()));
                (s.id.clone(), best.geo(&s.name, None, None))
            })
            .collect();
        Ok((items, format!("triangles {}", what.join(", "))))
    })
}

fn to_mesh_command(e: &Event) -> Reply {
    let delete = e.flag("mesh_delete_input", false);
    let r = built(
        e,
        "Mesh",
        "select surfaces or solids to convert, Enter",
        || {
            let objs = solids(e);
            let out: Vec<Geo> = objs
                .iter()
                .map(|s| Geo::Mesh {
                    name: format!("{} mesh", s.name),
                    vertices: s.mesh.as_ref().unwrap().vertices.clone(),
                    triangles: s.mesh.as_ref().unwrap().triangles.clone(),
                    meta: None,
                    layer: None,
                })
                .collect();
            let n: usize = objs
                .iter()
                .map(|s| s.mesh.as_ref().unwrap().triangles.len())
                .sum();
            Ok((out, format!("{n} triangles")))
        },
    );
    if r.done && delete {
        let mut r = r;
        r.delete = solids(e).iter().map(|s| s.id.clone()).collect();
        return r;
    }
    r
}

// ---- 3D printing, surface inset --------------------------------------------------------------

fn printing(e: &Event) -> Reply {
    let count = e.number("pr_sprues", 3.0).round().clamp(1.0, 12.0) as usize;
    let d = e.number("pr_diameter", 1.2);
    let len = e.number("pr_length", 4.0);
    let base = e.flag("pr_base", true);
    built(
        e,
        "3D Printing Supports",
        "select the ring (or model), Enter",
        || {
            let objs = solids(e);
            let s = objs.first().ok_or("select the model")?;
            let m = s.mesh.as_ref().unwrap();
            let b = to_mesh(m).bbox();
            let bottom = b[0][2];
            // Sprues under the lowest part, spread across its width.
            let mut out = Vec::new();
            let low: Vec<Vec3> = m
                .vertices
                .iter()
                .copied()
                .filter(|v| v[2] < bottom + (b[1][2] - bottom) * 0.1)
                .collect();
            let (x0, x1) = low
                .iter()
                .fold((f64::MAX, f64::MIN), |a, v| (a.0.min(v[0]), a.1.max(v[0])));
            for k in 0..count {
                let x = if count == 1 {
                    (x0 + x1) / 2.0
                } else {
                    x0 + (x1 - x0) * (k as f64 + 0.5) / count as f64
                };
                let tip = low
                    .iter()
                    .copied()
                    .min_by(|a, c| (a[0] - x).abs().total_cmp(&(c[0] - x).abs()))
                    .unwrap_or([x, 0.0, bottom]);
                let top = add(tip, [0.0, 0.0, d * 0.3]);
                let foot = [tip[0], tip[1], bottom - len];
                if let Some(p) = mesh::pipe(&[foot, top], false, d / 2.0, 16) {
                    out.push(p.geo("Sprue", None, Some("Supports")));
                }
            }
            if base {
                let c = [(b[0][0] + b[1][0]) / 2.0, (b[0][1] + b[1][1]) / 2.0];
                let pad = mesh::cuboid(
                    [b[0][0] - 2.0, c[1] - 3.0, bottom - len - 1.0],
                    [b[1][0] + 2.0, c[1] + 3.0, bottom - len],
                );
                out.push(pad.geo("Support base", None, Some("Supports")));
            }
            let n = out.len();
            Ok((out, format!("{n} support pieces")))
        },
    )
}

fn surface_inset(e: &Event) -> Reply {
    let depth = e.number("si_depth", 0.5);
    let inset = e.number("si_inset", 0.0);
    replaced(
        e,
        "Surface Inset",
        "select the solid and a closed curve on its surface, Enter",
        || {
            let objs = solids(e);
            let s = objs.first().ok_or("select the solid")?;
            let c = curves(e)
                .into_iter()
                .find(|c| c.closed)
                .ok_or("select a closed curve on the surface")?;
            let m = s.mesh.as_ref().unwrap();
            let n = e.plane.normal();
            let t = Target::new(m, n).ok_or("surface")?;
            // The pocket: the curve (inset in the CPlane) dropped onto the surface, cut down by
            // `depth` along the CPlane normal.
            let flat: Vec<[f64; 2]> = c
                .points
                .iter()
                .map(|&p| {
                    let l = e.plane.local(p);
                    [l[0], l[1]]
                })
                .collect();
            let o = if inset.abs() > 0.0 {
                mesh::offset_outline(&flat, -inset)
            } else {
                flat
            };
            let top: Vec<Vec3> = o
                .iter()
                .filter_map(|q| t.drop(e.plane.at(q[0], q[1], 1e6)).map(|h| h.0))
                .collect();
            if top.len() < 3 {
                return Err("the curve is not over the surface".into());
            }
            let zmax = top.iter().map(|p| dot(*p, n)).fold(f64::MIN, f64::max);
            let zmin = top.iter().map(|p| dot(*p, n)).fold(f64::MAX, f64::min);
            let base: Vec<Vec3> = o
                .iter()
                .map(|q| e.plane.at(q[0], q[1], zmin - depth))
                .collect();
            let tool =
                shapes::prism(&base, &[], scale(n, zmax - zmin + depth + 1.0)).ok_or("pocket")?;
            let cut = csg::boolean(&to_mesh(m), &tool, Op::Difference);
            Ok((
                vec![(s.id.clone(), cut.geo(&s.name, None, None))],
                format!("pocket {} mm deep", fmt(depth)),
            ))
        },
    )
}

// ---- object on curve, curve transform, center line -------------------------------------------

fn object_on_curve(e: &Event) -> Reply {
    let mode = e.text("ooc_mode", "count").to_owned();
    let count = e.number("ooc_count", 10.0).round().max(1.0) as usize;
    let spacing = e.number("ooc_spacing", 2.0);
    let scale_end = e.number("ooc_end_scale", 100.0) / 100.0;
    let twist = e.number("ooc_rotation", 0.0).to_radians();
    let cs: Vec<&Selected> = curves(e);
    let objs: Vec<&Selected> = e.selection.iter().filter(|s| s.kind != "curve").collect();
    let label = "Object on Curve";
    let Some(c) = cs.first().filter(|_| !objs.is_empty()) else {
        let r = Reply::ask(
            format!(
                "{label}: select the curve and the objects (modelled at the CPlane origin), Enter"
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a curve and objects"))
        } else {
            r
        };
    };
    let Some(path) = Path::new(&c.points, c.closed) else {
        return Reply::ask(label, json!({}));
    };
    let len = path.length();
    let stations: Vec<f64> = if mode == "spacing" {
        let n = (len / spacing.max(0.01)).floor() as usize + usize::from(!c.closed);
        (0..n).map(|k| k as f64 * spacing).collect()
    } else if c.closed {
        (0..count).map(|k| len * k as f64 / count as f64).collect()
    } else {
        (0..count)
            .map(|k| {
                if count == 1 {
                    len / 2.0
                } else {
                    len * k as f64 / (count - 1) as f64
                }
            })
            .collect()
    };
    let from = Frame::new(e.plane.origin, e.plane.normal(), e.plane.x);
    let mats: Vec<[f64; 16]> = stations
        .iter()
        .enumerate()
        .map(|(k, &s)| {
            let t = if stations.len() > 1 {
                k as f64 / (stations.len() - 1) as f64
            } else {
                0.0
            };
            let f = path.frame(s, Some(e.plane.normal())).rotated(twist * t);
            frame_matrix(&from, &f, 1.0 + (scale_end - 1.0) * t)
        })
        .collect();
    let n = mats.len();
    if matches!(e.input, Input::Enter) {
        return Reply::finish_transform(TransformOp {
            matrices: mats,
            keep_original: true,
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

/// Bend a curve like wire: points after the bend point (arc-length fraction `bend`) rotate
/// by `angle` about the axis through the bend point; points before the hold point stay.
pub fn bent(points: &[Vec3], hold: f64, bend: f64, angle: f64, axis: Vec3) -> Vec<Vec3> {
    let Some(p) = Path::new(points, false) else {
        return points.to_vec();
    };
    let len = p.length();
    let (h, b) = (hold.min(bend) * len, bend.max(hold) * len);
    let pivot = p.at(b);
    points
        .iter()
        .map(|&q| {
            let s = p.closest(q);
            // Full turn after the bend point, easing in between the hold and bend points.
            let w = if s >= b {
                1.0
            } else if s <= h || b - h < 1e-9 {
                0.0
            } else {
                (s - h) / (b - h)
            };
            let a = angle * w;
            let m = om9_addon_abi::session::rotation(if s >= b { pivot } else { p.at(h) }, axis, a);
            apply(&m, q)
        })
        .collect()
}

fn curve_transform(e: &Event) -> Reply {
    let hold = e.number("ct_hold", 40.0) / 100.0;
    let bend = e.number("ct_bend", 50.0) / 100.0;
    let angle = e.number("ct_angle", 30.0).to_radians();
    replaced(e, "Curve Transform", "select curves, Enter", || {
        let cs = curves(e);
        if cs.is_empty() {
            return Err("select a curve".into());
        }
        let items: Vec<(String, Geo)> = cs
            .iter()
            .filter(|c| !c.closed)
            .filter_map(|c| {
                smooth(&bent(&c.points, hold, bend, angle, e.plane.normal()), false)
                    .map(|g| (c.id.clone(), g))
            })
            .collect();
        let n = items.len();
        Ok((
            items,
            format!("{n} curves bent {}°", fmt(angle.to_degrees())),
        ))
    })
}

fn center_line(e: &Event) -> Reply {
    built(
        e,
        "Center Line",
        "select two curves (or one closed curve), Enter",
        || {
            let cs = curves(e);
            match cs.len() {
                2 => {
                    let (a, b) = (
                        Path::new(&cs[0].points, cs[0].closed).ok_or("curve")?,
                        Path::new(&cs[1].points, cs[1].closed).ok_or("curve")?,
                    );
                    let flip = dist(a.at(0.0), b.at(0.0)) > dist(a.at(0.0), b.at(b.length()));
                    let n = 64;
                    let pts: Vec<Vec3> = (0..=n)
                        .map(|k| {
                            let t = k as f64 / n as f64;
                            let tb = if flip { 1.0 - t } else { t };
                            scale(add(a.at(t * a.length()), b.at(tb * b.length())), 0.5)
                        })
                        .collect();
                    Ok((
                        smooth(&pts, false).into_iter().collect(),
                        "middle line between the curves".into(),
                    ))
                }
                1 => {
                    let c = cs[0];
                    let cen = scale(
                        c.points.iter().fold([0.0; 3], |s, &p| add(s, p)),
                        1.0 / c.points.len() as f64,
                    );
                    let l = e.plane.local(cen);
                    let pl = |u: f64, v: f64| e.plane.at(l[0] + u, l[1] + v, l[2]);
                    let ext = c
                        .points
                        .iter()
                        .map(|&p| {
                            let q = e.plane.local(p);
                            (q[0] - l[0]).abs().max((q[1] - l[1]).abs())
                        })
                        .fold(0.0, f64::max)
                        * 1.2;
                    Ok((
                        vec![
                            Geo::Line {
                                start: pl(-ext, 0.0),
                                end: pl(ext, 0.0),
                            },
                            Geo::Line {
                                start: pl(0.0, -ext),
                                end: pl(0.0, ext),
                            },
                        ],
                        "center cross".into(),
                    ))
                }
                _ => Err("select one or two curves".into()),
            }
        },
    )
}

#[cfg(test)]
mod tests;
