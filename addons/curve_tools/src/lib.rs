// SPDX-License-Identifier: LGPL-2.1-or-later
//! The rest of the Matrix-style **Curve** menu for OM9 (the drawing commands are in `curves`):
//! rebuild and refit, blends (quick, adjustable, arc), curves from two views, project,
//! pullback, intersections, offsets (planar, on surface, normal to surface), isocurves,
//! fillets and chamfers, UV curves, through points, extend, mark start/end, seam, continue,
//! divide, curve boolean, tween, duplicate edge/border, cross sections, sketches, curves on
//! surfaces, curve to lines, section, match, silhouette, wireframe and soft edit.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Plane, Replacement, Reply, Selected, Vec3, Want,
    add, cross, dist, dot, fmt, norm, scale, sub, unit,
};
use om9_addon_geom::pick::CurveRef;
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save};
use om9_addon_geom::nurbs::{self, Srf};
use om9_addon_geom::path::{Path, Target};
use om9_addon_geom::{Piece, composite_quadratic, interpolate, mesh, section, shapes};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const REBUILD: &str = "OthersCurveRebuild";
pub const REFIT: &str = "CurveCurveEditToolsRefitToTolerance";
pub const BLEND: &str = "CurveBlendCurvesBlendCurves";
pub const BLEND_ADJ: &str = "CurveBlendCurvesBlendCrv";
pub const CRV2VIEW: &str = "CurveCurveFrom2Views";
pub const CRV2VIEW_HISTORY: &str = "BuilderCrv2ViewHistory";
pub const PULLBACK: &str = "CurveCurveFromObjectsPullback";
pub const PROJECT: &str = "CurveCurveFromObjectsProject";
pub const INTERSECTION: &str = "CurveCurveFromObjectsIntersection";
pub const INTERSECT_TWO: &str = "CurveCurveFromObjectsIntersectTwoSets";
pub const OFFSET: &str = "CurveOffsetCurve";
pub const OFFSET_ON_SRF: &str = "CurveOffsetOffsetCrvOnSrf";
pub const OFFSET_NORMAL: &str = "CurveOffsetOffsetCrvNormalToSurface";
pub const ISOCURVE: &str = "CurveCurveFromObjectsExtractIsocurve";
pub const ANGLE_ISO: &str = "CurveGVExtractIsocurve";
pub const FILLET: &str = "CurveFilletCurves";
pub const FILLET_CORNERS: &str = "CurveFilletCorners";
pub const CHAMFER: &str = "CurveChamferCurves";
pub const UV_CREATE: &str = "CurveCurveFromObjectsCreateUVCurves";
pub const UV_APPLY: &str = "CurveCurveFromObjectsApplyUVCurves";
pub const THROUGH_POINTS: &str = "CurveFreeFormThroughPoints";
pub const EXTEND: &str = "CurveExtendCurveExtendCurve";
pub const MARK_START: &str = "CurvePointObjectMarkCurveStart";
pub const MARK_END: &str = "CurvePointObjectMarkCurveEnd";
pub const SEAM: &str = "CurveCurveEditToolsAdjustClosedCurveSeam";
pub const CONTINUE_INTERP: &str = "CurveFreeFormContinueInterpCurve";
pub const CONTINUE_CURVE: &str = "CurveFreeFormContinueCurve";
pub const DIVIDE: &str = "OthersCurveDivideCurve";
pub const CURVE_BOOLEAN: &str = "CurveCurveEditToolsCurveBoolean";
pub const TWEEN: &str = "CurveTweenCurves";
pub const DUP_EDGE: &str = "CurveCurveFromObjectsDuplicateEdge";
pub const DUP_BORDER: &str = "CurveCurveFromObjectsDuplicateBorder";
pub const CROSS_SECTION: &str = "CurveCrossSectionProfiles";
pub const SKETCH: &str = "CurveFreeFormSketch";
pub const SKETCH_SRF: &str = "CurveFreeFormSketchonSurface";
pub const SKETCH_MESH: &str = "CurveFreeFormSketchonPolygonMesh";
pub const POLY_ON_SRF: &str = "BuilderPolylineOnSurface";
pub const INTERP_ON_SRF: &str = "CurveFreeFormInterpolateonSurface";
pub const TO_LINES: &str = "CurveConvertCurveToLines";
pub const SECTION: &str = "CurveCurveFromObjectsSection";
pub const MATCH: &str = "CurveCurveEditToolsMatch";
pub const SILHOUETTE: &str = "CurveCurveFromObjectsSilhouette";
pub const WIREFRAME: &str = "CurveCurveFromObjectsExtractWireframe";
pub const SOFT_EDIT: &str = "CurveCurveEditToolsSoftEdit";
pub const ARC_BLEND: &str = "CurveBlendCurvesArcBlend";

mod blend;
mod edit;
mod extend;
mod fillet;
mod objects;
mod offset;
mod regions;
mod sections;

pub struct CurveTools;

impl Interactive for CurveTools {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        REBUILD,
        REFIT,
        BLEND,
        BLEND_ADJ,
        CRV2VIEW,
        CRV2VIEW_HISTORY,
        PULLBACK,
        PROJECT,
        INTERSECTION,
        INTERSECT_TWO,
        OFFSET,
        OFFSET_ON_SRF,
        OFFSET_NORMAL,
        ISOCURVE,
        ANGLE_ISO,
        FILLET,
        FILLET_CORNERS,
        CHAMFER,
        UV_CREATE,
        UV_APPLY,
        THROUGH_POINTS,
        EXTEND,
        MARK_START,
        MARK_END,
        SEAM,
        CONTINUE_INTERP,
        CONTINUE_CURVE,
        DIVIDE,
        CURVE_BOOLEAN,
        TWEEN,
        DUP_EDGE,
        DUP_BORDER,
        CROSS_SECTION,
        SKETCH,
        SKETCH_SRF,
        SKETCH_MESH,
        POLY_ON_SRF,
        INTERP_ON_SRF,
        TO_LINES,
        SECTION,
        MATCH,
        SILHOUETTE,
        WIREFRAME,
        SOFT_EDIT,
        ARC_BLEND,
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

// ---- helpers ---------------------------------------------------------------------------------

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

fn meshes(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind != "curve" && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty()))
        .collect()
}

fn surfaces(e: &Event) -> Vec<(&Selected, Srf<'_>)> {
    e.selection
        .iter()
        .filter_map(|s| s.exact.as_ref().and_then(Srf::of).map(|f| (s, f)))
        .collect()
}

/// Dense samples of a selected curve: the exact curve when known, else OM9's samples.
fn dense(s: &Selected, n: usize) -> Vec<Vec3> {
    match &s.exact {
        Some(g @ (Geo::Line { .. } | Geo::Circle { .. } | Geo::Arc { .. } | Geo::Nurbs { .. })) => {
            let mut p = nurbs::samples(g, n);
            if s.closed && p.len() > 2 && dist(p[0], *p.last().unwrap()) < 1e-9 {
                p.pop();
            }
            p
        }
        _ => s.points.clone(),
    }
}

/// A smooth curve through points (a polyline when there are only two).
fn smooth(points: &[Vec3], closed: bool) -> Option<Geo> {
    if points.len() == 2 && !closed {
        return Some(Geo::Line {
            start: points[0],
            end: points[1],
        });
    }
    interpolate(points, 3, closed)
}

fn polyline(points: Vec<Vec3>, closed: bool) -> Geo {
    Geo::Polyline { points, closed }
}

fn ask_objects(prompt: String) -> Reply {
    Reply::ask(prompt, json!({})).want(Want::Objects)
}

/// Selection-only command: prompt until Enter, then `run`.
fn on_enter(e: &Event, prompt: &str, run: impl FnOnce() -> Result<Reply, String>) -> Reply {
    if !matches!(e.input, Input::Enter) {
        return ask_objects(prompt.to_owned());
    }
    match run() {
        Ok(r) => r,
        Err(w) => ask_objects(prompt.to_owned()).refuse(w),
    }
}

fn finish_curves(label: &str, geo: Vec<Geo>) -> Result<Reply, String> {
    if geo.is_empty() {
        return Err(format!("{label}: nothing to create"));
    }
    let n = geo.len();
    Ok(Reply::finish(geo).say(format!("{label}: {n} curves")))
}

fn replace(label: &str, items: Vec<(String, Geo)>) -> Result<Reply, String> {
    if items.is_empty() {
        return Err(format!("{label}: select curves"));
    }
    let n = items.len();
    let mut r = Reply::finish(Vec::new()).say(format!("{label}: {n} curves changed"));
    r.replace = items
        .into_iter()
        .map(|(id, geo)| Replacement { id, geo })
        .collect();
    Ok(r)
}

/// Resample a polyline to `n` points evenly by arc length.
fn resample(points: &[Vec3], closed: bool, n: usize) -> Vec<Vec3> {
    let Some(p) = Path::new(points, closed) else {
        return points.to_vec();
    };
    let n = n.max(2);
    let len = p.length();
    if closed {
        (0..n).map(|i| p.at(len * i as f64 / n as f64)).collect()
    } else {
        (0..n)
            .map(|i| p.at(len * i as f64 / (n - 1) as f64))
            .collect()
    }
}

// ---- curves from views, projections ---------------------------------------------------------

fn from_two_views(e: &Event) -> Reply {
    let flow = Flow::new("Crv2View")
        .curve("Select first curve")
        .curve("Select second curve");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (a, _) = c.curve(0).ok_or("Select first curve")?;
        let (b, _) = c.curve(1).ok_or("Select second curve")?;
        let samples = |cv: &om9_addon_geom::pick::CurveRef, n: usize| -> Vec<Vec3> {
            let m = if cv.closed { n } else { n + 1 };
            (0..m).map(|i| cv.at(i as f64 / n as f64)).collect()
        };
        // The flatter curve (least spread along the CPlane normal) is the top view.
        let spread = |cv: &om9_addon_geom::pick::CurveRef| {
            let z: Vec<f64> = samples(cv, 64).iter().map(|&p| e.plane.local(p)[2]).collect();
            z.iter().cloned().fold(f64::MIN, f64::max) - z.iter().cloned().fold(f64::MAX, f64::min)
        };
        let (top, front) = if spread(a) <= spread(b) { (a, b) } else { (b, a) };
        let fr: Vec<Vec3> = samples(front, 400).iter().map(|&p| e.plane.local(p)).collect();
        let height_at = |x: f64| -> Option<f64> {
            fr.windows(2).find_map(|w| {
                let (a, b) = (w[0], w[1]);
                ((a[0] - x) * (b[0] - x) <= 0.0 && (b[0] - a[0]).abs() > 1e-12)
                    .then(|| a[2] + (b[2] - a[2]) * (x - a[0]) / (b[0] - a[0]))
            })
        };
        let pts: Vec<Vec3> = samples(top, 200)
            .iter()
            .filter_map(|&p| {
                let l = e.plane.local(p);
                height_at(l[0]).map(|z| e.plane.at(l[0], l[1], z))
            })
            .collect();
        if pts.len() < 2 {
            return Err("The two curves must overlap along the CPlane x axis".into());
        }
        finish_curves("Crv2View", smooth(&pts, top.closed && pts.len() > 3).into_iter().collect())
    })
}

// ---- intersections ---------------------------------------------------------------------------

fn chains_to_geo(chains: Vec<(Vec<Vec3>, bool)>) -> Vec<Geo> {
    chains.into_iter().map(|(p, c)| polyline(p, c)).collect()
}

/// Rhino 5 Intersect / IntersectTwoSets: points where curves cross curves or surfaces,
/// curves where surfaces / solids / meshes cross each other. `pairs` are the selection
/// index pairs to test.
fn intersect_objects(sel: &[&Selected], pairs: &[(usize, usize)]) -> Vec<Geo> {
    let mut points: Vec<Vec3> = Vec::new();
    let mut out = Vec::new();
    let solid = |s: &Selected| s.kind != "curve" && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty());
    for &(i, j) in pairs {
        let (a, b) = (sel[i], sel[j]);
        match (CurveRef::of(a), CurveRef::of(b)) {
            (Some(ca), Some(cb)) => points.extend(curve_curve_points(&ca, &cb)),
            (Some(c), None) if solid(b) => points.extend(curve_mesh_points(&c, b)),
            (None, Some(c)) if solid(a) => points.extend(curve_mesh_points(&c, a)),
            (None, None) if solid(a) && solid(b) => {
                out.extend(chains_to_geo(section::mesh_intersection(
                    a.mesh.as_ref().unwrap(),
                    b.mesh.as_ref().unwrap(),
                )));
            }
            _ => {}
        }
    }
    let mut kept: Vec<Vec3> = Vec::new();
    for p in points {
        if !kept.iter().any(|q| dist(*q, p) < 1e-6) {
            kept.push(p);
        }
    }
    out.extend(kept.into_iter().map(|point| Geo::Point { point }));
    out
}

/// Where two curves meet (within 1e-6): close segment pairs of dense samples, refined by
/// alternating closest points.
fn curve_curve_points(a: &CurveRef, b: &CurveRef) -> Vec<Vec3> {
    let n = 400;
    let pa: Vec<Vec3> = (0..=n).map(|k| a.at(k as f64 / n as f64)).collect();
    let pb: Vec<Vec3> = (0..=n).map(|k| b.at(k as f64 / n as f64)).collect();
    let seg = |p: &[Vec3], k: usize| dist(p[k], p[k + 1]);
    let mut out = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let reach = seg(&pa, i) + seg(&pb, j);
            let (m1, m2) = (scale(add(pa[i], pa[i + 1]), 0.5), scale(add(pb[j], pb[j + 1]), 0.5));
            if dist(m1, m2) > reach {
                continue;
            }
            let mut p = m1;
            let mut q = m2;
            for _ in 0..60 {
                q = b.closest(p).1;
                p = a.closest(q).1;
            }
            if dist(p, q) < 1e-6 && !out.iter().any(|r: &Vec3| dist(*r, p) < 1e-6) {
                out.push(scale(add(p, q), 0.5));
            }
        }
    }
    out
}

/// Where a curve passes through a surface, solid or mesh (its render mesh; the exact
/// surface refines the point when there is one).
fn curve_mesh_points(c: &CurveRef, s: &Selected) -> Vec<Vec3> {
    let m = s.mesh.as_ref().unwrap();
    let srf = s.exact.as_ref().and_then(Srf::of);
    let n = 800;
    let pts: Vec<Vec3> = (0..=n).map(|k| c.at(k as f64 / n as f64)).collect();
    let mut out = Vec::new();
    for k in 0..n {
        let (p0, p1) = (pts[k], pts[k + 1]);
        let d = sub(p1, p0);
        for t in &m.triangles {
            let [v0, v1, v2] = [0, 1, 2].map(|i| m.vertices[t[i] as usize]);
            let (e1, e2) = (sub(v1, v0), sub(v2, v0));
            let h = cross(d, e2);
            let det = dot(e1, h);
            if det.abs() < 1e-14 {
                continue;
            }
            let f = 1.0 / det;
            let sv = sub(p0, v0);
            let u = f * dot(sv, h);
            let q = cross(sv, e1);
            let v = f * dot(d, q);
            let w = f * dot(e2, q);
            if u < -1e-9 || v < -1e-9 || u + v > 1.0 + 1e-9 || !(-1e-9..=1.0 + 1e-9).contains(&w) {
                continue;
            }
            let mut hit = add(p0, scale(d, w));
            if let Some(f) = &srf {
                // Refine: the curve point nearest the surface point nearest it.
                for _ in 0..30 {
                    let (u, v) = f.closest_st(hit);
                    hit = c.closest(f.at_st(u, v)).1;
                }
            }
            if !out.iter().any(|r: &Vec3| dist(*r, hit) < 1e-6) {
                out.push(hit);
            }
        }
    }
    out
}

fn intersection(e: &Event) -> Reply {
    on_enter(
        e,
        "Select objects to intersect. Press Enter when done",
        || {
            let sel: Vec<&Selected> = e.selection.iter().collect();
            if sel.len() < 2 {
                return Err("select at least two objects".into());
            }
            let pairs: Vec<(usize, usize)> = (0..sel.len())
                .flat_map(|i| (i + 1..sel.len()).map(move |j| (i, j)))
                .collect();
            finish_curves("Intersection", intersect_objects(&sel, &pairs))
        },
    )
}

#[derive(Default, Serialize, Deserialize)]
struct FirstSet {
    /// The first set, by id.
    #[serde(default)]
    first: Vec<String>,
}

fn intersect_two_sets(e: &Event) -> Reply {
    let mut st: FirstSet = load(e);
    if st.first.is_empty() {
        if matches!(e.input, Input::Enter) && !e.selection.is_empty() {
            st.first = e.selection.iter().map(|s| s.id.clone()).collect();
            return Reply::ask(
                "Select second set of objects. Press Enter when done",
                save(&st),
            )
            .want(Want::Objects);
        }
        return Reply::ask("Select first set of objects. Press Enter when done", save(&st))
            .want(Want::Objects);
    }
    let sel: Vec<&Selected> = e.selection.iter().collect();
    let first: Vec<usize> = (0..sel.len()).filter(|&i| st.first.contains(&sel[i].id)).collect();
    let second: Vec<usize> = (0..sel.len()).filter(|&i| !st.first.contains(&sel[i].id)).collect();
    if !matches!(e.input, Input::Enter) || second.is_empty() {
        return Reply::ask(
            "Select second set of objects. Press Enter when done".to_owned(),
            save(&st),
        )
        .want(Want::Objects);
    }
    let pairs: Vec<(usize, usize)> = first
        .iter()
        .flat_map(|&i| second.iter().map(move |&j| (i, j)))
        .collect();
    finish_curves("Intersect Two Sets", intersect_objects(&sel, &pairs))
        .unwrap_or_else(|w| Reply::ask("Select second set of objects. Press Enter when done", save(&st)).want(Want::Objects).refuse(w))
}

// ---- offsets ---------------------------------------------------------------------------------

/// Offset a curve in the CPlane by `d` toward `side` (a point), keeping corners.
pub fn offset_points(
    points: &[Vec3],
    closed: bool,
    plane: &Plane,
    d: f64,
    side: Option<Vec3>,
) -> Vec<Vec3> {
    let flat: Vec<[f64; 2]> = points
        .iter()
        .map(|&p| {
            let l = plane.local(p);
            [l[0], l[1]]
        })
        .collect();
    let z: Vec<f64> = points.iter().map(|&p| plane.local(p)[2]).collect();
    let n = flat.len();
    let normal_at = |i: usize| -> [f64; 2] {
        let (a, b) = if closed {
            (flat[(i + n - 1) % n], flat[(i + 1) % n])
        } else {
            (flat[i.saturating_sub(1)], flat[(i + 1).min(n - 1)])
        };
        let t = [b[0] - a[0], b[1] - a[1]];
        let l = (t[0] * t[0] + t[1] * t[1]).sqrt().max(1e-15);
        [t[1] / l, -t[0] / l]
    };
    // Which side: the right-hand normal, flipped when the side point lies left.
    let mut sign = 1.0;
    if let Some(sp) = side {
        let l = plane.local(sp);
        let (i, _) = flat
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let da = (a.1[0] - l[0]).powi(2) + (a.1[1] - l[1]).powi(2);
                let db = (b.1[0] - l[0]).powi(2) + (b.1[1] - l[1]).powi(2);
                da.total_cmp(&db)
            })
            .unwrap();
        let nn = normal_at(i);
        if (l[0] - flat[i][0]) * nn[0] + (l[1] - flat[i][1]) * nn[1] < 0.0 {
            sign = -1.0;
        }
    }
    let off = if closed {
        let o = if shapes::area2(&flat) > 0.0 {
            mesh::offset_outline(&flat, d * sign)
        } else {
            mesh::offset_outline(&flat, -d * sign)
        };
        // offset_outline pushes along the outward normal of a counter-clockwise outline.
        let ccw = shapes::area2(&flat) > 0.0;
        let _ = ccw;
        o
    } else {
        (0..n)
            .map(|i| {
                let nn = normal_at(i);
                [flat[i][0] + nn[0] * d * sign, flat[i][1] + nn[1] * d * sign]
            })
            .collect()
    };
    off.iter()
        .zip(&z)
        .map(|(q, &h)| plane.at(q[0], q[1], h))
        .collect()
}

// ---- isocurves -------------------------------------------------------------------------------

fn isocurve(e: &Event) -> Reply {
    let angle_mode = e.command == ANGLE_ISO;
    let label = if angle_mode {
        "Extract Angle Isocurve"
    } else {
        "Extract Isocurve"
    };
    let mut flow = Flow::new(label);
    if angle_mode {
        flow = flow.number("Angle of the section (degrees from the CPlane x axis)", 0.0);
    } else {
        flow = flow.choice("Direction", &["U", "V", "Both"], "U");
    }
    let flow = flow.point("Point on the surface for the isocurve");
    let make = |c: &Collected, p: Vec3| -> Vec<Geo> {
        let mut out = Vec::new();
        if !angle_mode {
            for (_, s) in surfaces(e) {
                let (u, v) = s.closest_st(p);
                match c.opt(&flow, "Direction") {
                    "V" => out.push(s.isocurve(1, v)),
                    "Both" => {
                        out.push(s.isocurve(0, u));
                        out.push(s.isocurve(1, v));
                    }
                    _ => out.push(s.isocurve(0, u)),
                }
            }
            if !out.is_empty() {
                return out;
            }
        }
        // Meshes: a section through the point, perpendicular to the CPlane at the angle.
        let a = c.n(0).unwrap_or(0.0).to_radians();
        let along = add(scale(e.plane.x, a.cos()), scale(e.plane.y, a.sin()));
        let sec_n = unit(cross(along, e.plane.normal()));
        for s in meshes(e) {
            out.extend(chains_to_geo(section::section(
                s.mesh.as_ref().unwrap(),
                p,
                sec_n,
            )));
        }
        out
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: h.map(|p| make(c, p)).unwrap_or_default(),
            ..Default::default()
        },
        &|c| finish_curves(label, make(c, c.p(0).unwrap())),
    )
}

// ---- fillets and chamfers --------------------------------------------------------------------

/// Corner geometry: from the corner `x` the two legs leave along `u` and `v`.
fn fillet_arc(x: Vec3, u: Vec3, v: Vec3, r: f64) -> Option<(Vec3, Vec3, Vec3)> {
    let (u, v) = (unit(u), unit(v));
    let half = dot(u, v).clamp(-1.0, 1.0).acos() / 2.0;
    if half < 1e-6 || (std::f64::consts::FRAC_PI_2 - half) < 1e-9 {
        return None;
    }
    let t = r / half.tan();
    let a = add(x, scale(u, t));
    let b = add(x, scale(v, t));
    let bis = unit(add(u, v));
    let center = add(x, scale(bis, r / half.sin()));
    Some((a, b, center))
}

/// A polyline with every corner rounded by `r` (exact lines and arcs).
pub fn round_corners(points: &[Vec3], closed: bool, r: f64) -> Option<Geo> {
    let n = points.len();
    if n < 3 {
        return None;
    }
    let mut pieces = Vec::new();
    let corner = |i: usize| -> Option<(Vec3, Vec3, Vec3)> {
        let (p, x, q) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
        let (u, v) = (sub(p, x), sub(q, x));
        let max_r = norm(u).min(norm(v)) / 2.0;
        let half = dot(unit(u), unit(v)).clamp(-1.0, 1.0).acos() / 2.0;
        let r_fit = r.min(max_r * half.tan());
        fillet_arc(x, u, v, r_fit)
    };
    let range: Vec<usize> = if closed {
        (0..n).collect()
    } else {
        (1..n - 1).collect()
    };
    let corners: Vec<Option<(Vec3, Vec3, Vec3)>> = range.iter().map(|&i| corner(i)).collect();
    let mut cursor = if closed {
        corners[0].map(|c| c.1).unwrap_or(points[0])
    } else {
        points[0]
    };
    let iter: Vec<usize> = if closed {
        (1..=n).collect()
    } else {
        (0..corners.len()).collect()
    };
    for k in iter {
        let (idx, cr) = if closed {
            (k % n, corners[k % n])
        } else {
            (range[k], corners[k])
        };
        match cr {
            Some((a, b, center)) => {
                if dist(cursor, a) > 1e-9 {
                    pieces.push(Piece::Line(cursor, a));
                }
                // Arcs up to 90°: split bigger turns in two.
                let mid_dir = unit(sub(add(scale(add(a, b), 0.5), [0.0; 3]), center));
                let rr = dist(a, center);
                let m = add(center, scale(mid_dir, rr));
                let ang = dot(unit(sub(a, center)), unit(sub(b, center)))
                    .clamp(-1.0, 1.0)
                    .acos();
                if ang > std::f64::consts::FRAC_PI_2 {
                    pieces.push(Piece::Arc {
                        center,
                        start: a,
                        end: m,
                    });
                    pieces.push(Piece::Arc {
                        center,
                        start: m,
                        end: b,
                    });
                } else {
                    pieces.push(Piece::Arc {
                        center,
                        start: a,
                        end: b,
                    });
                }
                cursor = b;
            }
            None => {
                let x = points[idx];
                if dist(cursor, x) > 1e-9 {
                    pieces.push(Piece::Line(cursor, x));
                }
                cursor = x;
            }
        }
    }
    if !closed && dist(cursor, points[n - 1]) > 1e-9 {
        pieces.push(Piece::Line(cursor, points[n - 1]));
    }
    composite_quadratic(&pieces)
}

// ---- UV curves -------------------------------------------------------------------------------

fn uv_size(s: &Srf) -> (f64, f64) {
    let len = |g: &Geo| {
        let p = nurbs::samples(g, 64);
        p.windows(2)
            .map(|w| dist(w[0], w[1]))
            .sum::<f64>()
            .max(1e-6)
    };
    (len(&s.isocurve(1, 0.5)), len(&s.isocurve(0, 0.5)))
}

fn uv_create(e: &Event) -> Reply {
    on_enter(
        e,
        "Select surface to create UV curves (and curves on it). Press Enter when done",
        || {
            let srfs = surfaces(e);
            let (_, s) = srfs.first().ok_or("select a NURBS surface")?;
            let (w, h) = uv_size(s);
            let o = e.plane.origin;
            let at = |u: f64, v: f64| {
                e.plane.at(
                    e.plane.local(o)[0] + u * w,
                    e.plane.local(o)[1] + v * h,
                    0.0,
                )
            };
            let mut out = vec![polyline(
                vec![at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)],
                true,
            )];
            for c in curves(e) {
                let uv: Vec<Vec3> = dense(c, 120)
                    .iter()
                    .map(|&p| {
                        let (u, v) = s.closest_st(p);
                        at(u, v)
                    })
                    .collect();
                out.extend(smooth(&uv, c.closed));
            }
            finish_curves("Create UV Curves", out)
        },
    )
}

fn uv_apply(e: &Event) -> Reply {
    on_enter(
        e,
        "Select curves to apply and the surface to apply them to. Press Enter when done",
        || {
            let srfs = surfaces(e);
            let (_, s) = srfs.first().ok_or("select a NURBS surface")?;
            let (w, h) = uv_size(s);
            let o = e.plane.local(e.plane.origin);
            let mut out = Vec::new();
            for c in curves(e) {
                let pts: Vec<Vec3> = dense(c, 200)
                    .iter()
                    .map(|&p| {
                        let l = e.plane.local(p);
                        s.at_st((l[0] - o[0]) / w, (l[1] - o[1]) / h)
                    })
                    .collect();
                out.extend(smooth(&pts, c.closed));
            }
            finish_curves("Apply UV Curves", out)
        },
    )
}

// ---- points and simple curve edits ---------------------------------------------------------

/// CurveThroughPt: the selected point objects become the control points of a curve (in the
/// order they lie along, nearest neighbour from the outermost).
fn through_points(e: &Event) -> Reply {
    let flow = Flow::new("CurveThroughPt")
        .select_prompt("Select points for control points of curve")
        .num("Degree", 3.0)
        .choice("CurveType", &["ControlPoint", "Interpolated"], "ControlPoint")
        .choice("Knots", &["Uniform", "Chord", "SqrtChrd"], "Chord");
    let make = |c: &Collected| -> Result<Geo, String> {
        let pts: Vec<Vec3> = e
            .selection
            .iter()
            .filter(|s| s.kind == "point")
            .map(|s| match &s.exact {
                Some(Geo::Point { point }) => *point,
                _ => scale(add(s.bbox[0], s.bbox[1]), 0.5),
            })
            .collect();
        if pts.len() < 2 {
            return Err("Select two or more points".into());
        }
        let degree = c.num(&flow, "Degree").round();
        if !(1.0..=11.0).contains(&degree) {
            return Err("Degree is a whole number from 1 to 11".into());
        }
        let centroid = scale(pts.iter().fold([0.0; 3], |a, &p| add(a, p)), 1.0 / pts.len() as f64);
        let mut rest = pts.clone();
        let start = (0..rest.len())
            .max_by(|&a, &b| dist(rest[a], centroid).total_cmp(&dist(rest[b], centroid)))
            .unwrap();
        let mut order = vec![rest.remove(start)];
        while !rest.is_empty() {
            let last = *order.last().unwrap();
            let i = (0..rest.len())
                .min_by(|&a, &b| dist(rest[a], last).total_cmp(&dist(rest[b], last)))
                .unwrap();
            order.push(rest.remove(i));
        }
        // Rhino: ControlPoint puts the control points on the points, Interpolated makes the
        // curve pass through them (with the Knots spacing).
        let curve = if c.opt(&flow, "CurveType").eq_ignore_ascii_case("Interpolated") {
            let knots = om9_addon_geom::interp::Knots::parse(c.opt(&flow, "Knots"))
                .unwrap_or(om9_addon_geom::interp::Knots::Chord);
            om9_addon_geom::interp::interpolate(
                &order,
                degree as usize,
                knots,
                None,
                None,
                om9_addon_geom::interp::Closing::Open,
            )
        } else {
            om9_addon_geom::control_curve(&order, degree as usize)
        };
        curve.ok_or_else(|| "No curve through these points".to_owned())
    };
    flow.step(
        e,
        &|c, _| Show { geo: make(c).into_iter().collect(), ..Default::default() },
        &|c| finish_curves("CurveThroughPt", vec![make(c)?]),
    )
}

fn mark(e: &Event) -> Reply {
    let end = e.command == MARK_END;
    let label = if end {
        "Mark Curve End"
    } else {
        "Mark Curve Start"
    };
    let prompt = if end { "Select curves for end point. Press Enter when done" } else { "Select curves for start point. Press Enter when done" };
    on_enter(e, prompt, || {
        let mut out = Vec::new();
        for s in curves(e) {
            let p = dense(s, 50);
            let (a, b) = if end {
                (p[p.len() - 1], p[p.len() - 2])
            } else {
                (p[0], p[1])
            };
            let _ = b;
            out.push(Geo::Point { point: a });
        }
        if out.is_empty() {
            return Err("select curves".into());
        }
        let n = out.len();
        Ok(Reply::finish(out).say(format!("{label}: {n} points")))
    })
}

fn seam(e: &Event) -> Reply {
    let flow = Flow::new("Adjust Closed Curve Seam")
        .select_prompt("Select closed curves for seam adjustment")
        .point("New seam point");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let p = c.p(0).unwrap();
        let items = curves(e)
            .iter()
            .filter(|s| s.closed)
            .filter_map(|s| {
                let mut pts = match &s.exact {
                    Some(Geo::Nurbs {
                        degree: 1, points, ..
                    }) => {
                        let mut q = points.clone();
                        q.pop();
                        q
                    }
                    _ => resample(&dense(s, 300), true, 64),
                };
                let path = Path::new(&pts, true)?;
                let s_at = path.closest(p);
                let seam_pt = path.at(s_at);
                let i = (0..pts.len())
                    .min_by(|&a, &b| dist(pts[a], seam_pt).total_cmp(&dist(pts[b], seam_pt)))?;
                pts.rotate_left(i);
                if dist(pts[0], seam_pt) > 1e-9 {
                    pts.insert(0, seam_pt);
                }
                let g = if matches!(&s.exact, Some(Geo::Nurbs { degree: 1, .. })) {
                    polyline(pts, true)
                } else {
                    smooth(&pts, true)?
                };
                Some((s.id.clone(), g))
            })
            .collect();
        replace("Adjust Closed Curve Seam", items)
    })
}

#[derive(Default, Serialize, Deserialize)]
struct Clicks {
    points: Vec<Vec3>,
    /// Sketch: `OnSurface=Yes` / `OnMesh=Yes`.
    #[serde(default)]
    on_surface: bool,
    #[serde(default)]
    on_mesh: bool,
    /// Sketch (Rhino 5): `Closed=Yes` closes the stroke, `Planar=Yes` keeps it flat.
    #[serde(default)]
    closed: bool,
    #[serde(default)]
    planar: bool,
}

fn continue_curve(e: &Event) -> Reply {
    let interp = e.command == CONTINUE_INTERP;
    let label = if interp {
        "Continue Interp Curve"
    } else {
        "Continue Control Point Curve"
    };
    let mut st: Clicks = load(e);
    let cs = curves(e);
    let Some(s) = cs.first() else {
        return ask_objects("Select curve to continue near end".to_owned());
    };
    match &e.input {
        Input::Point { point } => st.points.push(*point),
        Input::Undo => {
            st.points.pop();
        }
        Input::Enter if !st.points.is_empty() => {
            // Continue from the end picked (the start: the curve runs backwards first).
            let from_start = om9_addon_geom::pick::CurveRef::picked(e, &[])
                .and_then(|c| c.1)
                .is_some_and(|t| t < 0.5);
            let g = if interp {
                let mut pts = resample(&dense(s, 300), false, 16);
                if from_start {
                    pts.reverse();
                }
                pts.extend(&st.points);
                smooth(&pts, false)
            } else {
                let (degree, mut poles) = match &s.exact {
                    Some(Geo::Nurbs { degree, points, .. }) => (*degree, points.clone()),
                    _ => (3, resample(&s.points, false, 8)),
                };
                if from_start {
                    poles.reverse();
                }
                poles.extend(&st.points);
                om9_addon_geom::control_curve(&poles, degree)
            };
            return match g {
                Some(g) => {
                    let mut r = Reply::finish(Vec::new())
                        .say(format!("{label}: {} points added", st.points.len()));
                    r.replace.push(Replacement {
                        id: s.id.clone(),
                        geo: g,
                    });
                    r
                }
                None => Reply::ask(label, save(&st)).refuse("curve"),
            };
        }
        _ => {}
    }
    let from_start = om9_addon_geom::pick::CurveRef::picked(e, &[])
        .and_then(|c| c.1)
        .is_some_and(|t| t < 0.5);
    let ends = dense(s, 50);
    let last = st
        .points
        .last()
        .copied()
        .unwrap_or_else(|| if from_start { ends[0] } else { *ends.last().unwrap() });
    let mut opts = Vec::new();
    if !st.points.is_empty() {
        opts.push(om9_addon_abi::session::OptionToken::action("Undo"));
    }
    Reply::ask("Next point. Press Enter when done", save(&st))
        .with_options(opts)
        .want(Want::Point)
        .rubber(Some(last))
}

/// Curves from meshes and surfaces: Duplicate Edge (sharp edges and borders), Duplicate
/// Border, Silhouette (seen along the CPlane normal), Extract Wireframe.
fn from_objects(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        DUP_EDGE => "Duplicate Edge",
        DUP_BORDER => "Duplicate Border",
        SILHOUETTE => "Silhouette",
        _ => "Extract Wireframe",
    };
    let prompt = match cmd {
        DUP_EDGE => "Select edges to duplicate. Press Enter when done",
        DUP_BORDER => "Select surface or polysurface for border duplication. Press Enter when done",
        SILHOUETTE => "Select objects for silhouette creation. Press Enter when done",
        _ => "Select surfaces, polysurfaces or meshes for wireframe extraction. Press Enter when done",
    };
    on_enter(
        e,
        prompt,
        || {
            let mut out = Vec::new();
            for (_, s) in surfaces(e) {
                match cmd {
                    WIREFRAME => {
                        for k in 0..=4 {
                            let t = k as f64 / 4.0;
                            out.push(s.isocurve(0, t));
                            out.push(s.isocurve(1, t));
                        }
                    }
                    SILHOUETTE => {}
                    _ => {
                        for t in [0.0, 1.0] {
                            out.push(s.isocurve(0, t));
                            out.push(s.isocurve(1, t));
                        }
                    }
                }
            }
            let exact_ids: Vec<&str> = surfaces(e).iter().map(|(s, _)| s.id.as_str()).collect();
            for s in meshes(e) {
                if exact_ids.contains(&s.id.as_str()) && cmd != SILHOUETTE {
                    continue;
                }
                let m = s.mesh.as_ref().unwrap();
                let segs = match cmd {
                    DUP_EDGE => section::feature_edges(m, 30.0, false),
                    DUP_BORDER => section::feature_edges(m, 0.0, true),
                    SILHOUETTE => section::silhouette(m, e.plane.normal()),
                    _ => section::feature_edges(m, 10.0, false),
                };
                out.extend(chains_to_geo(section::chain(&segs, 1e-7)));
            }
            finish_curves(label, out)
        },
    )
}

/// The Sketch commands take Rhino 5's `Closed` (and on the plain Sketch `Planar`,
/// `OnSurface`, `OnMesh`) before the stroke starts.
fn sketch_options(cmd: &str) -> bool {
    matches!(cmd, SKETCH | SKETCH_SRF | SKETCH_MESH)
}

/// Sketches and curves through clicked points, optionally on a surface.
fn draw_on(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        SKETCH => "Sketch",
        SKETCH_SRF => "Sketch on Surface",
        SKETCH_MESH => "Sketch on Polygon Mesh",
        POLY_ON_SRF => "Polyline on Surface",
        _ => "Interpolate on Surface",
    };
    let mut st: Clicks = load(e);
    if sketch_options(cmd)
        && let Input::Option { name, value } = &e.input
    {
        let flag = |cur: bool| match value.as_deref() {
            Some(v) => v.eq_ignore_ascii_case("yes"),
            None => !cur,
        };
        match name.as_str() {
            "OnSurface" => st.on_surface = flag(st.on_surface),
            "OnMesh" => st.on_mesh = flag(st.on_mesh),
            "Closed" => st.closed = flag(st.closed),
            "Planar" => st.planar = flag(st.planar),
            _ => {}
        }
    }
    let on_surface = cmd != SKETCH || st.on_surface || st.on_mesh;
    let sketch = matches!(cmd, SKETCH | SKETCH_SRF | SKETCH_MESH);
    let n = e.plane.normal();
    let ms = meshes(e);
    let target = ms
        .first()
        .and_then(|s| Target::new(s.mesh.as_ref().unwrap(), n));
    if on_surface && target.is_none() {
        let prompt = match cmd {
            SKETCH_MESH => "Select mesh to sketch on",
            POLY_ON_SRF => "Select surface for polyline",
            INTERP_ON_SRF => "Select surface to draw curve on",
            _ if st.on_mesh => "Select mesh to sketch on",
            _ => "Select surface to sketch on",
        };
        return Reply::ask(prompt, save(&st)).want(Want::Objects);
    }
    let drop = |p: Vec3| -> Vec3 {
        target
            .as_ref()
            .and_then(|t| t.drop(add(p, scale(n, 1e6))))
            .map_or(p, |h| h.0)
    };
    // Sketch follows the pointer: the first click starts the stroke, moving draws it, the
    // next click (or Enter) ends it.
    let mut end_stroke = false;
    match &e.input {
        Input::Hover { point } if sketch && !st.points.is_empty() => {
            let p = drop(*point);
            if st.points.last().is_some_and(|q| dist(*q, p) >= 0.1) {
                st.points.push(p);
            }
        }
        Input::Point { point } if sketch && !st.points.is_empty() => {
            st.points.push(drop(*point));
            end_stroke = true;
        }
        _ => {}
    }
    let input = if end_stroke { Input::Enter } else { e.input.clone() };
    match &input {
        Input::Hover { .. } => {}
        Input::Point { point } => st.points.push(drop(*point)),
        Input::Undo => {
            st.points.pop();
        }
        Input::Enter if st.points.len() >= 2 => {
            let pts = st.points.clone();
            let g = match cmd {
                POLY_ON_SRF => {
                    // Each segment follows the surface.
                    let mut dense_pts = Vec::new();
                    for w in pts.windows(2) {
                        for i in 0..16 {
                            dense_pts
                                .push(drop(add(w[0], scale(sub(w[1], w[0]), i as f64 / 16.0))));
                        }
                    }
                    dense_pts.push(*pts.last().unwrap());
                    Some(polyline(dense_pts, false))
                }
                SKETCH | SKETCH_SRF | SKETCH_MESH => {
                    // Smooth the stroke a little, then fit with about one pole per 4 points.
                    let mut s = pts.clone();
                    for _ in 0..2 {
                        let prev = s.clone();
                        for i in 1..s.len() - 1 {
                            s[i] = scale(
                                add(add(prev[i - 1], prev[i + 1]), scale(prev[i], 2.0)),
                                0.25,
                            );
                        }
                    }
                    if st.planar && !on_surface {
                        // Planar: every point in the plane of the first point (CPlane normal).
                        let o = s[0];
                        for q in s.iter_mut() {
                            *q = sub(*q, scale(n, dot(sub(*q, o), n)));
                        }
                    }
                    let closed = st.closed && s.len() >= 4;
                    if closed && dist(s[0], *s.last().unwrap()) < 1e-9 {
                        s.pop();
                    }
                    let poles = (s.len() / 4).clamp(4, 64).min(s.len());
                    let fitted = if s.len() > 8 {
                        om9_addon_geom::interp::fit(&s, poles, 3, closed).or_else(|| smooth(&s, closed))
                    } else {
                        smooth(&s, closed)
                    };
                    if on_surface {
                        fitted.map(|g| {
                            let p: Vec<Vec3> =
                                nurbs::samples(&g, 120).into_iter().map(drop).collect();
                            smooth(&p, false).unwrap_or(g)
                        })
                    } else {
                        fitted
                    }
                }
                _ => smooth(&pts, false).map(|g| {
                    let p: Vec<Vec3> = nurbs::samples(&g, 120).into_iter().map(drop).collect();
                    smooth(&p, false).unwrap_or(g)
                }),
            };
            return match g {
                Some(g) => Reply::finish(vec![g]).say(format!("{label}: {} points", pts.len())),
                None => Reply::ask(label, save(&st)).refuse("points"),
            };
        }
        _ => {}
    }
    let preview = if st.points.len() >= 2 {
        vec![polyline(st.points.clone(), false)]
    } else {
        Vec::new()
    };
    let started = !st.points.is_empty();
    let prompt = match (sketch, cmd == POLY_ON_SRF, started) {
        (true, _, false) => "Start of sketch",
        (true, _, true) => "Move to draw, click to end the sketch",
        (false, true, false) => "Start of polyline",
        (false, true, true) => "Next point of polyline. Press Enter when done",
        (false, false, false) => "Start of curve",
        (false, false, true) => "Next point. Press Enter when done",
    };
    let mut opts = Vec::new();
    if sketch_options(cmd) && !started {
        opts.push(om9_addon_abi::session::OptionToken::toggle("Closed", st.closed));
        if cmd == SKETCH {
            opts.push(om9_addon_abi::session::OptionToken::toggle("Planar", st.planar));
            opts.push(om9_addon_abi::session::OptionToken::toggle("OnSurface", st.on_surface));
            opts.push(om9_addon_abi::session::OptionToken::toggle("OnMesh", st.on_mesh));
        }
    }
    if started {
        opts.push(om9_addon_abi::session::OptionToken::action("Undo"));
    }
    let r = Reply::ask(prompt, save(&st))
        .with_options(opts)
        .want(Want::Point)
        .preview(preview);
    if sketch {
        r.tracking()
    } else {
        r.rubber(st.points.last().copied())
    }
}

#[cfg(test)]
mod tests;

/// Helper for tests: a planar polygon offset keeps its distance.
pub fn fmt_len(v: f64) -> String {
    fmt(v)
}

/// Commands whose reply to a pointer move never reads the pointer (Rebuild, Refit to tolerance).
const STEADY_PREVIEW: &[&str] = &[REBUILD, REFIT];

fn dispatch(e: &Event) -> Result<Reply, String> {
    Ok(match e.command.as_str() {
        REBUILD => edit::rebuild(e),
        REFIT => edit::fit_curve(e),
        BLEND => blend::blend(e, false),
        BLEND_ADJ => blend::blend(e, true),
        ARC_BLEND => blend::arc_blend(e),
        CRV2VIEW | CRV2VIEW_HISTORY => from_two_views(e),
        PULLBACK | PROJECT => objects::project(e, e.command == PULLBACK),
        INTERSECTION => intersection(e),
        INTERSECT_TWO => intersect_two_sets(e),
        OFFSET => offset::offset(e),
        OFFSET_ON_SRF => objects::offset_on_surface(e),
        OFFSET_NORMAL => objects::offset_normal(e),
        ISOCURVE => objects::isocurve(e),
        ANGLE_ISO => isocurve(e),
        FILLET | CHAMFER => fillet::fillet(e, e.command == CHAMFER),
        FILLET_CORNERS => fillet::fillet_corners(e),
        UV_CREATE => uv_create(e),
        UV_APPLY => uv_apply(e),
        THROUGH_POINTS => through_points(e),
        EXTEND => extend::extend(e),
        MARK_START | MARK_END => mark(e),
        SEAM => seam(e),
        CONTINUE_INTERP | CONTINUE_CURVE => continue_curve(e),
        DIVIDE => edit::divide(e),
        CURVE_BOOLEAN => regions::curve_boolean(e),
        TWEEN => edit::tween(e),
        DUP_EDGE | DUP_BORDER | SILHOUETTE | WIREFRAME => from_objects(e),
        CROSS_SECTION => sections::csec(e),
        SKETCH | SKETCH_SRF | SKETCH_MESH | POLY_ON_SRF | INTERP_ON_SRF => draw_on(e),
        TO_LINES => edit::convert(e),
        SECTION => sections::section_cmd(e),
        MATCH => blend::match_curve(e),
        SOFT_EDIT => objects::soft_edit(e),
        other => return Err(format!("unknown command {other}")),
    })
}
