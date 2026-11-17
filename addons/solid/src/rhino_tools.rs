// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5 Slab, MakeHole and Rib (as closed meshes, like the rest of this menu's own
//! commands; OM9's exact solids stand in for the ones it has).
//!
//! - **Slab**: Select curve, Side to offset (Distance, ThroughPoint, BothSides), Height: the
//!   curve offset in the CPlane (mitered corners), the band between the curve and its offset
//!   extruded.
//! - **MakeHole**: closed curves and the solid, then the depth point or Enter to cut through
//!   (Direction X / Y / Z / NormalToCurve / CPlaneNormal, BothSides, DeleteInput).
//! - **Rib**: planar curves and the boundary, Offset InCurvePlane / NormalToCurvePlane,
//!   Distance, Mode Straight / Tapered with DraftAngle: the curve thickened and run on to the
//!   boundary.
use super::*;
use om9_addon_abi::session::{cross, norm, unit};

/// The CPlane as a 2D frame: to (x, y, height) and back.
struct Flat {
    o: Vec3,
    x: Vec3,
    y: Vec3,
    n: Vec3,
}

impl Flat {
    fn of(e: &Event) -> Flat {
        Flat { o: e.plane.origin, x: unit(e.plane.x), y: unit(e.plane.y), n: e.plane.normal() }
    }
    fn to2(&self, p: Vec3) -> [f64; 2] {
        let d = sub(p, self.o);
        [dot(d, self.x), dot(d, self.y)]
    }
    fn h(&self, p: Vec3) -> f64 {
        dot(sub(p, self.o), self.n)
    }
    fn to3(&self, q: [f64; 2], h: f64) -> Vec3 {
        add(add(add(self.o, scale(self.x, q[0])), scale(self.y, q[1])), scale(self.n, h))
    }
}

fn clean(pts: &[[f64; 2]], closed: bool) -> Vec<[f64; 2]> {
    let mut v: Vec<[f64; 2]> = Vec::new();
    for p in pts {
        if v.last().is_none_or(|q: &[f64; 2]| (q[0] - p[0]).hypot(q[1] - p[1]) > 1e-9) {
            v.push(*p);
        }
    }
    if closed && v.len() > 2 {
        let (a, b) = (v[0], v[v.len() - 1]);
        if (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9 {
            v.pop();
        }
    }
    v
}

fn area2(p: &[[f64; 2]]) -> f64 {
    (0..p.len()).map(|i| {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        a[0] * b[1] - b[0] * a[1]
    }).sum::<f64>() / 2.0
}

/// A 2D polyline offset by `d` to its left (negative: right), with mitered corners (the
/// miter capped at four times the distance, as a sharp spike would be).
pub(crate) fn offset2(p: &[[f64; 2]], closed: bool, d: f64) -> Vec<[f64; 2]> {
    let n = p.len();
    let left = |a: [f64; 2], b: [f64; 2]| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = dx.hypot(dy).max(1e-300);
        [-dy / l, dx / l]
    };
    (0..n)
        .map(|i| {
            let prev = if i > 0 { Some(left(p[i - 1], p[i])) } else if closed { Some(left(p[n - 1], p[0])) } else { None };
            let next = if i + 1 < n { Some(left(p[i], p[i + 1])) } else if closed { Some(left(p[n - 1], p[0])) } else { None };
            let m = match (prev, next) {
                (Some(a), Some(b)) => {
                    let s = [a[0] + b[0], a[1] + b[1]];
                    let l = s[0].hypot(s[1]);
                    if l < 1e-12 {
                        a
                    } else {
                        let u = [s[0] / l, s[1] / l];
                        let c = (u[0] * a[0] + u[1] * a[1]).max(0.25);
                        [u[0] / c, u[1] / c]
                    }
                }
                (Some(a), None) | (None, Some(a)) => a,
                (None, None) => [0.0, 0.0],
            };
            [p[i][0] + m[0] * d, p[i][1] + m[1] * d]
        })
        .collect()
}

/// The nearest place on a 2D polyline to `q`: distance and which side (left > 0).
fn nearest2(p: &[[f64; 2]], closed: bool, q: [f64; 2]) -> (f64, f64) {
    let n = p.len();
    let segs = if closed { n } else { n - 1 };
    let mut best = (f64::MAX, 1.0);
    for i in 0..segs {
        let (a, b) = (p[i], p[(i + 1) % n]);
        let ab = [b[0] - a[0], b[1] - a[1]];
        let l2 = (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-300);
        let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1]) / l2).clamp(0.0, 1.0);
        let c = [a[0] + ab[0] * t, a[1] + ab[1] * t];
        let d = (q[0] - c[0]).hypot(q[1] - c[1]);
        if d < best.0 {
            let side = ab[0] * (q[1] - a[1]) - ab[1] * (q[0] - a[0]);
            best = (d, if side >= 0.0 { 1.0 } else { -1.0 });
        }
    }
    best
}

/// The slab's base region in the CPlane: (outline, holes).
fn slab_region(p: &[[f64; 2]], closed: bool, d: f64, both: bool) -> (Vec<[f64; 2]>, Vec<Vec<[f64; 2]>>) {
    let (a, b) = if both { (offset2(p, closed, d.abs()), offset2(p, closed, -d.abs())) } else { (p.to_vec(), offset2(p, closed, d)) };
    if closed {
        if area2(&a).abs() >= area2(&b).abs() { (a, vec![b]) } else { (b, vec![a]) }
    } else {
        let mut out = a;
        out.extend(b.into_iter().rev());
        (out, Vec::new())
    }
}

/// Rhino 5 Slab: Select curve, Side to offset (Distance, ThroughPoint, BothSides), Height.
pub(crate) fn slab(e: &Event) -> Reply {
    let f = Flat::of(e);
    let flow = Flow::new("Slab")
        .select_prompt("Select curve")
        .point("Side to offset")
        .value("Height", e.number("sl_height", 1.0))
        .num("Distance", e.number("sl_distance", 1.0))
        .toggle("ThroughPoint", false)
        .toggle("BothSides", false);
    let curve = || -> Result<(&Selected, Vec<[f64; 2]>, f64), String> {
        let c = curves(e).into_iter().next().ok_or("select a curve")?;
        let p = clean(&c.points.iter().map(|q| f.to2(*q)).collect::<Vec<_>>(), c.closed);
        if p.len() < 2 {
            return Err("the curve is too short".into());
        }
        let base = c.points.iter().map(|q| f.h(*q)).sum::<f64>() / c.points.len() as f64;
        Ok((c, p, base))
    };
    // The signed offset from the side point.
    let offset = |c: &Collected, side: Vec3| -> Result<f64, String> {
        let (s, p, _) = curve()?;
        let (d, sign) = nearest2(&p, s.closed, f.to2(side));
        let dist = if c.yes(&flow, "ThroughPoint") { d } else { c.num(&flow, "Distance") };
        if dist <= 0.0 {
            return Err("the distance must be more than 0".into());
        }
        Ok(sign * dist)
    };
    let make = |c: &Collected, side: Vec3, height: f64| -> Result<Mesh, String> {
        let (s, p, base) = curve()?;
        let d = offset(c, side)?;
        if height.abs() < 1e-12 {
            return Err("the height must not be 0".into());
        }
        let (outline, holes) = slab_region(&p, s.closed, d, c.yes(&flow, "BothSides"));
        let o3: Vec<Vec3> = outline.iter().map(|q| f.to3(*q, base)).collect();
        let h3: Vec<Vec<Vec3>> = holes.iter().map(|h| h.iter().map(|q| f.to3(*q, base)).collect()).collect();
        shapes::prism(&o3, &h3, scale(f.n, height)).ok_or_else(|| "the offset crosses itself".to_owned())
    };
    let height_of = |c: &Collected, k: usize| -> Option<f64> {
        match c.answers.get(k)? {
            om9_addon_geom::cmd::Answer::Number(v) => Some(*v),
            om9_addon_geom::cmd::Answer::Point(q) => {
                let base = curve().ok()?.2;
                Some(f.h(*q) - base)
            }
            _ => None,
        }
    };
    flow.step(
        e,
        &|c, hover| {
            let geo = match (c.p(0), hover) {
                (None, Some(side)) => offset(c, side).ok().and_then(|d| {
                    let (s, p, base) = curve().ok()?;
                    let (outline, holes) = slab_region(&p, s.closed, d, c.yes(&flow, "BothSides"));
                    let mut g = vec![om9_addon_geom::cmd::closed_polyline(outline.iter().map(|q| f.to3(*q, base)).collect())];
                    g.extend(holes.iter().map(|h| om9_addon_geom::cmd::closed_polyline(h.iter().map(|q| f.to3(*q, base)).collect())));
                    Some(g)
                }),
                (Some(side), Some(h)) => {
                    let base = curve().map(|x| x.2).unwrap_or(0.0);
                    make(c, side, f.h(h) - base).ok().map(|m| m.wire())
                }
                _ => None,
            };
            Show { geo: geo.unwrap_or_default(), ..Default::default() }
        },
        &|c| {
            let side = c.p(0).ok_or("pick the side")?;
            let h = height_of(c, 1).ok_or("the height")?;
            let m = make(c, side, h)?;
            let v = m.volume().abs();
            Ok(Reply::finish(vec![solid_geo(m, "Slab")]).say(format!("Slab: height {} mm, volume {} mm³", fmt(h.abs()), fmt(v))))
        },
    )
}

/// Möller–Trumbore: distance along `dir` (> 0) to the triangle, if hit.
fn ray_hit(o: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f64> {
    let (e1, e2) = (sub(b, a), sub(c, a));
    let pv = cross(dir, e2);
    let det = dot(e1, pv);
    if det.abs() < 1e-14 {
        return None;
    }
    let inv = 1.0 / det;
    let tv = sub(o, a);
    let u = dot(tv, pv) * inv;
    if !(-1e-12..=1.0 + 1e-12).contains(&u) {
        return None;
    }
    let qv = cross(tv, e1);
    let v = dot(dir, qv) * inv;
    if v < -1e-12 || u + v > 1.0 + 1e-12 {
        return None;
    }
    let t = dot(e2, qv) * inv;
    (t > 1e-9).then_some(t)
}

/// Nearest hit of a ray on a mesh (distance along `dir`).
pub(crate) fn cast_mesh(m: &SelectedMesh, o: Vec3, dir: Vec3) -> Option<f64> {
    cast(m, o, dir)
}

/// Nearest hit of a ray on a mesh.
fn cast(m: &SelectedMesh, o: Vec3, dir: Vec3) -> Option<f64> {
    m.triangles
        .iter()
        .filter_map(|t| ray_hit(o, dir, m.vertices[t[0] as usize], m.vertices[t[1] as usize], m.vertices[t[2] as usize]))
        .min_by(|a, b| a.total_cmp(b))
}

/// Rhino 5 MakeHole: closed curves and the solid, then the depth point (Enter: through).
/// Direction X / Y / Z / NormalToCurve / CPlaneNormal; BothSides; DeleteInput.
pub(crate) fn make_hole(e: &Event) -> Reply {
    let flow = Flow::new("Make Hole")
        .select_prompt("Select closed curves and the surface or polysurface")
        .value("Depth point. Press Enter to cut through", 0.0)
        .choice("Direction", &["CPlaneNormal", "NormalToCurve", "X", "Y", "Z"], "CPlaneNormal")
        .toggle("BothSides", false)
        .toggle("DeleteInput", false);
    let make = |c: &Collected, depth_pt: Option<Vec3>, typed: Option<f64>| -> Result<(String, Mesh, Vec<String>, String), String> {
        let objs = solids(e);
        let s = objs.first().ok_or("select the solid")?;
        let holes: Vec<&Selected> = curves(e).into_iter().filter(|c| c.closed).collect();
        if holes.is_empty() {
            return Err("select closed curves".into());
        }
        let size = dist(s.bbox[0], s.bbox[1]) + 1.0;
        let mut result = to_mesh(s.mesh.as_ref().unwrap());
        for h in &holes {
            let dir = match c.opt(&flow, "Direction") {
                "X" => [1.0, 0.0, 0.0],
                "Y" => [0.0, 1.0, 0.0],
                "Z" => [0.0, 0.0, 1.0],
                "NormalToCurve" => shapes::loop_plane(&h.points).map(|x| x.1).ok_or("the curve is not planar")?,
                _ => e.plane.normal(),
            };
            let o = h.points[0];
            // From the curve along the direction: to the depth point, by the typed depth, or
            // through (both ways, the whole solid).
            let (a, b) = match (depth_pt, typed) {
                (Some(q), _) => {
                    let d = dot(sub(q, o), dir);
                    if c.yes(&flow, "BothSides") { (-d.abs(), d.abs()) } else { (d.min(0.0), d.max(0.0)) }
                }
                (None, Some(d)) if d != 0.0 => {
                    if c.yes(&flow, "BothSides") { (-d.abs(), d.abs()) } else { (d.min(0.0), d.max(0.0)) }
                }
                _ => (-size, size),
            };
            if b - a < 1e-9 {
                return Err("the depth must not be 0".into());
            }
            let start: Vec<Vec3> = h.points.iter().map(|p| add(*p, scale(dir, a))).collect();
            let start = if h.points.len() > 1 && dist(start[0], *start.last().unwrap()) < 1e-9 { start[..start.len() - 1].to_vec() } else { start };
            let cutter = shapes::prism(&start, &[], scale(dir, b - a)).ok_or("the curve is not planar")?;
            result = csg::boolean(&result, &cutter, Op::Difference);
        }
        let del = if c.yes(&flow, "DeleteInput") { holes.iter().map(|h| h.id.clone()).collect() } else { Vec::new() };
        Ok((s.id.clone(), result, del, format!("Make Hole: {} holes", holes.len())))
    };
    flow.step(
        e,
        &|_, _| Show::default(),
        &|c| {
            let (q, typed) = match c.answers.first() {
                Some(om9_addon_geom::cmd::Answer::Point(q)) => (Some(*q), None),
                Some(om9_addon_geom::cmd::Answer::Number(v)) => (None, Some(*v)),
                _ => (None, None),
            };
            let (id, m, del, msg) = make(c, q, typed)?;
            let v = m.volume();
            Ok(finish_replace(&id, m, del, format!("{msg}, volume {} mm³", fmt(v))))
        },
    )
}

/// Rhino 5 Rib: planar curves and the boundary; Offset InCurvePlane (the curve is the rib
/// seen from above: it is thickened in its plane and run on along the plane's normal to the
/// boundary) or NormalToCurvePlane (seen from the side: run on in its plane, against the
/// CPlane normal, to the boundary, and thickened across its plane); Distance (thickness);
/// Mode Straight / Tapered with DraftAngle (the walls lean out by the angle).
pub(crate) fn rib(e: &Event) -> Reply {
    let flow = Flow::new("Rib")
        .select_prompt("Select planar curves to rib and the boundary")
        .confirm("Press Enter to make the rib, or change an option")
        .choice("Offset", &["InCurvePlane", "NormalToCurvePlane"], "InCurvePlane")
        .num("Distance", e.number("rb_distance", 1.0))
        .choice("Mode", &["Straight", "Tapered"], "Straight")
        .num("DraftAngle", e.number("rb_draft_angle", 0.0));
    let make = |c: &Collected| -> Result<Vec<Mesh>, String> {
        let objs = solids(e);
        let bound = objs.first().ok_or("select the boundary surface or polysurface")?;
        let bm = bound.mesh.as_ref().unwrap();
        let t = c.num(&flow, "Distance");
        if t <= 0.0 {
            return Err("the distance must be more than 0".into());
        }
        let tapered = c.opt(&flow, "Mode") == "Tapered";
        let draft = if tapered { c.num(&flow, "DraftAngle").to_radians() } else { 0.0 };
        let mut out = Vec::new();
        for cv in curves(e) {
            let centre_pn = shapes::loop_plane(&cv.points).or_else(|| {
                // An open planar curve: the plane through its ends and its farthest point.
                let a = cv.points[0];
                let b = *cv.points.last().unwrap();
                let far = cv.points.iter().copied().max_by(|p, q| norm(cross(sub(*p, a), sub(b, a))).total_cmp(&norm(cross(sub(*q, a), sub(b, a)))))?;
                let n = cross(sub(b, a), sub(far, a));
                (norm(n) > 1e-12).then(|| (a, unit(n)))
            });
            let in_plane = c.opt(&flow, "Offset") == "InCurvePlane";
            // A straight curve has no plane of its own: seen from above it lies in the CPlane;
            // seen from the side, in the upright plane through it.
            let (centre, pn) = centre_pn.unwrap_or_else(|| {
                let n = e.plane.normal();
                let along = sub(*cv.points.last().unwrap(), cv.points[0]);
                let side = cross(along, n);
                (cv.points[0], if in_plane || norm(side) < 1e-12 { n } else { unit(side) })
            });
            // A frame on the curve's plane.
            let x = unit(sub(cv.points[cv.points.len() / 2], centre)).map_or_zero(pn);
            let fr = Flat { o: centre, x, y: cross(pn, x), n: pn };
            let p2 = clean(&cv.points.iter().map(|q| fr.to2(*q)).collect::<Vec<_>>(), cv.closed);
            if p2.len() < 2 {
                return Err("a curve is too short".into());
            }
            if in_plane {
                // Thicken in the plane, then each point runs along ±normal to the boundary.
                let (outline, holes) = slab_region(&p2, cv.closed, t / 2.0, true);
                let dir = {
                    let up = cast(bm, centre, pn).unwrap_or(f64::MAX);
                    let down = cast(bm, centre, scale(pn, -1.0)).unwrap_or(f64::MAX);
                    if up.min(down) == f64::MAX {
                        return Err("the curve does not face the boundary".into());
                    }
                    if up <= down { pn } else { scale(pn, -1.0) }
                };
                let o3: Vec<Vec3> = outline.iter().map(|q| fr.to3(*q, 0.0)).collect();
                let h3: Vec<Vec<Vec3>> = holes.iter().map(|h| h.iter().map(|q| fr.to3(*q, 0.0)).collect()).collect();
                let unit_prism = shapes::prism(&o3, &h3, scale(dir, 1.0)).ok_or("the curve crosses itself")?;
                // Lift every top point to the boundary (and out by the draft).
                let mut m = unit_prism;
                let all: Vec<[f64; 2]> = outline.iter().chain(holes.iter().flatten()).copied().collect();
                let mid = all.iter().fold([0.0, 0.0], |s, q| [s[0] + q[0], s[1] + q[1]]);
                let mid = [mid[0] / all.len() as f64, mid[1] / all.len() as f64];
                for v in m.vertices.iter_mut() {
                    if dot(sub(*v, centre), dir) > 0.5 {
                        let bottom = sub(*v, dir);
                        let reach = cast(bm, bottom, dir).ok_or("part of the rib does not reach the boundary")?;
                        let mut top = add(bottom, scale(dir, reach));
                        if draft != 0.0 {
                            // Out, away from the rib's centre line, by height·tan(draft).
                            let q = fr.to2(bottom);
                            let (_, side) = nearest2(&p2, cv.closed, q);
                            let away = offset2(&p2, cv.closed, side).iter().copied().min_by(|a, b| {
                                (a[0] - q[0]).hypot(a[1] - q[1]).total_cmp(&(b[0] - q[0]).hypot(b[1] - q[1]))
                            });
                            let _ = mid;
                            if let Some(a) = away {
                                let base = p2.iter().copied().min_by(|a, b| (a[0] - q[0]).hypot(a[1] - q[1]).total_cmp(&(b[0] - q[0]).hypot(b[1] - q[1]))).unwrap();
                                let w = [a[0] - base[0], a[1] - base[1]];
                                let l = w[0].hypot(w[1]).max(1e-300);
                                let grow = reach * draft.tan();
                                top = add(top, add(scale(fr.x, w[0] / l * grow), scale(fr.y, w[1] / l * grow)));
                            }
                        }
                        *v = top;
                    }
                }
                out.push(m);
            } else {
                // Side view: each point runs against the CPlane normal (or along it) to the
                // boundary, within the curve's plane; then thickened across the plane.
                let down = scale(e.plane.normal(), -1.0);
                let hits: Vec<Option<(Vec3, Vec3)>> = cv
                    .points
                    .iter()
                    .map(|p| {
                        let a = cast(bm, *p, down).map(|d| (d, down));
                        let b = cast(bm, *p, scale(down, -1.0)).map(|d| (d, scale(down, -1.0)));
                        let (d, dir) = match (a, b) {
                            (Some(x), Some(y)) => if x.0 <= y.0 { x } else { y },
                            (Some(x), None) | (None, Some(x)) => x,
                            _ => return None,
                        };
                        Some((*p, add(*p, scale(dir, d))))
                    })
                    .collect();
                let hits: Option<Vec<(Vec3, Vec3)>> = hits.into_iter().collect();
                let hits = hits.ok_or("part of the rib does not reach the boundary")?;
                // The sheet's outline in the curve's plane: the curve, then back along the hits.
                let mut outline: Vec<Vec3> = hits.iter().map(|h| h.0).collect();
                outline.extend(hits.iter().rev().map(|h| h.1));
                let outline: Vec<Vec3> = outline.iter().map(|p| sub(*p, scale(pn, t / 2.0))).collect();
                out.push(shapes::prism(&outline, &[], scale(pn, t)).ok_or("the rib outline crosses itself")?);
            }
        }
        if out.is_empty() {
            return Err("select planar curves".into());
        }
        Ok(out)
    };
    flow.step(
        e,
        &|c, _| Show { geo: make(c).map(|ms| ms.iter().flat_map(|m| m.wire()).collect()).unwrap_or_default(), ..Default::default() },
        &|c| {
            let ms = make(c)?;
            let v: f64 = ms.iter().map(|m| m.volume().abs()).sum();
            let k = ms.len();
            Ok(Reply::finish(ms.into_iter().map(|m| solid_geo(m, "Rib")).collect()).say(format!("Rib: {k} ribs, volume {} mm³", fmt(v))))
        },
    )
}

trait OrZero {
    fn map_or_zero(self, n: Vec3) -> Vec3;
}

impl OrZero for Vec3 {
    /// A unit vector in the plane of normal `n` (this one made square to `n`, or any).
    fn map_or_zero(self, n: Vec3) -> Vec3 {
        let v = sub(self, scale(n, dot(self, n)));
        if norm(v) > 1e-9 {
            unit(v)
        } else {
            let a = if n[0].abs() < 0.9 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
            unit(cross(n, a))
        }
    }
}

/// Rhino 5 hole tools, "Select a hole": the hole whose edge is nearest the click — a loop of
/// the planar face clicked on (or next to) that is not the face's outer edge. The loop comes
/// back as points round it (on the face).
pub(crate) fn pick_hole(m: &SelectedMesh, click: Vec3) -> Option<Vec<Vec3>> {
    let w = shapes::weld(&to_mesh(m), 1e-7);
    let tri_n = |t: usize| {
        let tr = w.triangles[t];
        shapes::tri_normal(w.vertices[tr[0] as usize], w.vertices[tr[1] as usize], w.vertices[tr[2] as usize])
    };
    let size = w.vertices.iter().fold(0.0f64, |s, v| s.max(v[0].abs()).max(v[1].abs()).max(v[2].abs())).max(1.0);
    // Distance from the click to each triangle.
    let tri_dist = |t: usize| -> f64 {
        let tr = w.triangles[t];
        let (a, b, c) = (w.vertices[tr[0] as usize], w.vertices[tr[1] as usize], w.vertices[tr[2] as usize]);
        let n = tri_n(t);
        let foot = sub(click, scale(n, dot(sub(click, a), n)));
        let inside = [(a, b), (b, c), (c, a)].iter().all(|(x, y)| dot(cross(sub(*y, *x), sub(foot, *x)), n) >= -1e-12);
        if inside {
            return dot(sub(click, a), n).abs();
        }
        [(a, b), (b, c), (c, a)]
            .iter()
            .map(|(x, y)| {
                let ab = sub(*y, *x);
                let t = (dot(sub(click, *x), ab) / dot(ab, ab).max(1e-300)).clamp(0.0, 1.0);
                dist(click, add(*x, scale(ab, t)))
            })
            .fold(f64::MAX, f64::min)
    };
    let mut near: Vec<(f64, usize)> = (0..w.triangles.len()).map(|t| (tri_dist(t), t)).collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let reach = near.first()?.0 + 0.05 * size;
    let mut tried = std::collections::HashSet::new();
    for &(d, seed) in near.iter().take_while(|x| x.0 <= reach) {
        if tried.contains(&seed) {
            continue;
        }
        // The planar face through the seed: every triangle in its plane facing the same way
        // (by plane, not by shared edges: boolean results leave T-junctions).
        let n0 = tri_n(seed);
        let off0 = dot(w.vertices[w.triangles[seed][0] as usize], n0);
        let face: Vec<usize> = (0..w.triangles.len())
            .filter(|t| dot(tri_n(*t), n0) > 1.0 - 1e-9 && w.triangles[*t].iter().all(|v| (dot(w.vertices[*v as usize], n0) - off0).abs() < 1e-7))
            .collect();
        tried.extend(face.iter().copied());
        let _ = d;
        // Its edge loops: edges used once within the face, chained. Boolean results can
        // leave T-junctions (a vertex in the middle of a neighbour's edge): edges are split at
        // the face's vertices lying on them first, so inner edges cancel.
        let fverts: Vec<u32> = {
            let mut v: Vec<u32> = face.iter().flat_map(|t| w.triangles[*t]).collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        let split = |a: u32, b: u32| -> Vec<u32> {
            let (pa, pb) = (w.vertices[a as usize], w.vertices[b as usize]);
            let ab = sub(pb, pa);
            let l2 = dot(ab, ab);
            let mut on: Vec<(f64, u32)> = fverts
                .iter()
                .filter(|v| **v != a && **v != b)
                .filter_map(|v| {
                    let p = w.vertices[*v as usize];
                    let t = dot(sub(p, pa), ab) / l2.max(1e-300);
                    (t > 1e-9 && t < 1.0 - 1e-9 && dist(p, add(pa, scale(ab, t))) < 1e-7).then_some((t, *v))
                })
                .collect();
            on.sort_by(|x, y| x.0.total_cmp(&y.0));
            std::iter::once(a).chain(on.into_iter().map(|x| x.1)).chain(std::iter::once(b)).collect()
        };
        let mut count = std::collections::HashMap::<(u32, u32), (u32, u32)>::new();
        for &t in &face {
            let tr = w.triangles[t];
            for k in 0..3 {
                let chain = split(tr[k], tr[(k + 1) % 3]);
                for seg in chain.windows(2) {
                    let (a, b) = (seg[0], seg[1]);
                    let key = (a.min(b), a.max(b));
                    if count.remove(&key).is_none() {
                        count.insert(key, (a, b));
                    }
                }
            }
        }
        let mut next: std::collections::HashMap<u32, u32> = count.values().map(|(a, b)| (*a, *b)).collect();
        let mut loops: Vec<Vec<u32>> = Vec::new();
        while let Some((&start, _)) = next.iter().next() {
            let mut lp = vec![start];
            let mut cur = start;
            while let Some(nx) = next.remove(&cur) {
                if nx == start {
                    break;
                }
                lp.push(nx);
                cur = nx;
            }
            // Drop points in the middle of straight runs.
            let k = lp.len();
            let lp: Vec<u32> = (0..k)
                .filter(|i| {
                    let (p, q, r) = (w.vertices[lp[(i + k - 1) % k] as usize], w.vertices[lp[*i] as usize], w.vertices[lp[(i + 1) % k] as usize]);
                    norm(cross(sub(q, p), sub(r, q))) > 1e-9 * (dist(p, q) * dist(q, r)).max(1e-300).sqrt().max(1e-12)
                })
                .map(|i| lp[i])
                .collect();
            if lp.len() >= 3 {
                loops.push(lp);
            }
        }
        if loops.len() < 2 {
            continue;
        }
        // The outer edge encloses the most area; the rest are holes.
        let area = |lp: &[u32]| -> f64 {
            let mut s = [0.0; 3];
            for i in 0..lp.len() {
                let (p, q) = (w.vertices[lp[i] as usize], w.vertices[lp[(i + 1) % lp.len()] as usize]);
                s = add(s, cross(p, q));
            }
            dot(s, n0).abs() / 2.0
        };
        let outer = (0..loops.len()).max_by(|a, b| area(&loops[*a]).total_cmp(&area(&loops[*b])))?;
        let loop_dist = |lp: &[u32]| -> f64 {
            (0..lp.len())
                .map(|i| {
                    let (p, q) = (w.vertices[lp[i] as usize], w.vertices[lp[(i + 1) % lp.len()] as usize]);
                    let ab = sub(q, p);
                    let t = (dot(sub(click, p), ab) / dot(ab, ab).max(1e-300)).clamp(0.0, 1.0);
                    dist(click, add(p, scale(ab, t)))
                })
                .fold(f64::MAX, f64::min)
        };
        let best = (0..loops.len()).filter(|i| *i != outer).min_by(|a, b| loop_dist(&loops[*a]).total_cmp(&loop_dist(&loops[*b])))?;
        if loop_dist(&loops[best]) <= 0.05 * size {
            return Some(loops[best].iter().map(|i| w.vertices[*i as usize]).collect());
        }
    }
    None
}
