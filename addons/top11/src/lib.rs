// SPDX-License-Identifier: LGPL-2.1-or-later
//! The rest of Matrix's **Top 11** toolbar for OM9: edit points and control points (pick a
//! point, place it, repeat), explode, join, split, trim and ring rail. (Duplicate, mirror,
//! move and rotate are in the Transform crate.)
#![allow(clippy::type_complexity)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Replacement, Reply, Selected, SelectedMesh, Vec3, Want, add,
    cross, dist, dot, norm, scale, sub, unit,
};
use om9_addon_geom::cmd::{Flow, Show, load, save};
use om9_addon_geom::mesh::Mesh;
use om9_addon_geom::nurbs::Srf;
use om9_addon_geom::path::Path;
use om9_addon_geom::{interpolate, nurbs};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

pub const EDIT_POINTS: &str = "TopIconEditPointsOn";
pub const CONTROL_POINTS: &str = "TopIconControlPointsOn";
pub const EXPLODE: &str = "TopIconExplode";
pub const JOIN: &str = "TopIconJoin";
pub const SPLIT: &str = "TopIconSplit";
pub const TRIM: &str = "TopIconTrim";
pub const RING_RAIL: &str = "TopIconRingRail";

pub struct Top11;

impl Interactive for Top11 {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        EDIT_POINTS,
        CONTROL_POINTS,
        EXPLODE,
        JOIN,
        SPLIT,
        TRIM,
        RING_RAIL,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            EDIT_POINTS | CONTROL_POINTS => points(e),
            EXPLODE => explode(e),
            JOIN => join(e),
            SPLIT => split(e),
            TRIM => trim(e),
            RING_RAIL => ring_rail(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- points on ---------------------------------------------------------------------------

/// The editable points of an object and how to rebuild it from moved points.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Editable {
    /// Control points of a NURBS curve (degree, weights, knots kept).
    Curve {
        degree: usize,
        weights: Vec<f64>,
        knots: Vec<f64>,
    },
    /// Points the curve passes through (rebuilt by interpolation).
    Through { closed: bool },
    /// Polyline vertices.
    Polyline { closed: bool },
    /// Control points of a NURBS surface.
    Surface {
        degree: [usize; 2],
        size: [usize; 2],
        weights: Vec<f64>,
        knots: [Vec<f64>; 2],
    },
    /// Mesh vertices (triangles kept).
    Mesh { triangles: Vec<[u32; 3]> },
}

pub fn editable(s: &Selected, control: bool) -> Option<(Editable, Vec<Vec3>)> {
    match &s.exact {
        Some(Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        }) if control => Some((
            Editable::Curve {
                degree: *degree,
                weights: weights.clone(),
                knots: knots.clone(),
            },
            points.clone(),
        )),
        Some(Geo::Polyline { points, closed }) => {
            Some((Editable::Polyline { closed: *closed }, points.clone()))
        }
        Some(Geo::Line { start, end }) => {
            Some((Editable::Polyline { closed: false }, vec![*start, *end]))
        }
        Some(Geo::Surface {
            degree,
            size,
            points,
            weights,
            knots,
        }) => Some((
            Editable::Surface {
                degree: *degree,
                size: *size,
                weights: weights.clone(),
                knots: knots.clone(),
            },
            points.clone(),
        )),
        Some(g @ (Geo::Circle { .. } | Geo::Arc { .. } | Geo::Nurbs { .. })) => {
            // Edit points: about one per span, the curve rebuilt through them.
            let n = match g {
                Geo::Nurbs { points, .. } => points.len().max(4),
                _ => 8,
            };
            let closed = s.closed;
            let mut p = nurbs::samples(g, if closed { n } else { n - 1 });
            if closed && p.len() > 2 && dist(p[0], *p.last().unwrap()) < 1e-9 {
                p.pop();
            }
            Some((Editable::Through { closed }, p))
        }
        _ => {
            if let Some(m) = &s.mesh {
                return Some((
                    Editable::Mesh {
                        triangles: m.triangles.clone(),
                    },
                    m.vertices.clone(),
                ));
            }
            (s.kind == "curve" && s.points.len() >= 2).then(|| {
                (
                    Editable::Through { closed: s.closed },
                    om9_surfaces::resample(&s.points, s.closed, 10),
                )
            })
        }
    }
}

pub fn rebuild(kind: &Editable, pts: &[Vec3]) -> Option<Geo> {
    Some(match kind {
        Editable::Curve {
            degree,
            weights,
            knots,
        } => Geo::Nurbs {
            degree: *degree,
            points: pts.to_vec(),
            weights: weights.clone(),
            knots: knots.clone(),
        },
        Editable::Through { closed } => interpolate(pts, 3, *closed)?,
        Editable::Polyline { closed } => Geo::Polyline {
            points: pts.to_vec(),
            closed: *closed,
        },
        Editable::Surface {
            degree,
            size,
            weights,
            knots,
        } => Geo::Surface {
            degree: *degree,
            size: *size,
            points: pts.to_vec(),
            weights: weights.clone(),
            knots: knots.clone(),
        },
        Editable::Mesh { triangles } => Geo::Mesh {
            name: "Mesh".into(),
            vertices: pts.to_vec(),
            triangles: triangles.clone(),
            meta: None,
            layer: None,
        },
    })
}

#[derive(Default, Serialize, Deserialize)]
struct Editing {
    /// Per object: what it is and its points as edited so far.
    objects: Vec<(String, Editable, Vec<Vec3>)>,
    /// The point picked to move: (object, index).
    picked: Option<(usize, usize)>,
}

/// The point nearest to the line through `p` along `dir`.
fn nearest(
    objects: &[(String, Editable, Vec<Vec3>)],
    p: Vec3,
    dir: Vec3,
) -> Option<(usize, usize)> {
    let d = unit(dir);
    let mut best: Option<(f64, usize, usize, f64)> = None;
    for (o, (_, _, pts)) in objects.iter().enumerate() {
        for (i, q) in pts.iter().enumerate() {
            let c = sub(*q, p);
            let off = norm(sub(c, scale(d, dot(c, d))));
            // Points stacked along the view: the one nearest the viewer wins.
            let better = best.is_none_or(|b: (f64, usize, usize, f64)| {
                off < b.0 - 1e-9 || (off < b.0 + 1e-9 && dot(*q, d) > b.3 + 1e-9)
            });
            if better {
                best = Some((off, o, i, dot(*q, d)));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

fn points(e: &Event) -> Reply {
    let control = e.command == CONTROL_POINTS;
    let label = if control {
        "Control Points On"
    } else {
        "Edit Points On"
    };
    edit_points(e, control, label)
}

/// The point editor (also behind Solid / Transform "Solid Points On"): select objects, click a
/// point, click (or type) where it goes, repeat, Enter. `control` picks control points of
/// NURBS over points on the curve.
pub fn edit_points(e: &Event, control: bool, label: &str) -> Reply {
    let mut st: Editing = load(e);
    if st.objects.is_empty() {
        st.objects = e
            .selection
            .iter()
            .filter_map(|s| editable(s, control).map(|(k, p)| (s.id.clone(), k, p)))
            .collect();
        if st.objects.is_empty() {
            let r = Reply::ask(
                format!("{label}: select curves, surfaces or meshes, Enter"),
                json!({}),
            )
            .want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: nothing with points is selected"))
            } else {
                r
            };
        }
    }
    let dir = e.plane.normal();
    match &e.input {
        Input::Point { point } => match st.picked {
            None => st.picked = nearest(&st.objects, *point, dir),
            Some((o, i)) => {
                // Keep the point's height along the view: move it in the CPlane.
                let old = st.objects[o].2[i];
                let d = sub(*point, old);
                st.objects[o].2[i] = add(old, sub(d, scale(dir, dot(d, dir))));
                st.picked = None;
            }
        },
        Input::Text { text } => {
            // Typed coordinates place the picked point exactly.
            if let (Some((o, i)), Some(p)) = (
                st.picked,
                om9_addon_abi::session::parse_point(text, &e.plane, None),
            ) {
                st.objects[o].2[i] = p;
                st.picked = None;
            }
        }
        Input::Enter => {
            let mut r = Reply::finish(Vec::new())
                .say(format!("{label}: {} objects edited", st.objects.len()));
            for (id, kind, pts) in &st.objects {
                if let Some(g) = rebuild(kind, pts) {
                    r.replace.push(Replacement {
                        id: id.clone(),
                        geo: g,
                    });
                }
            }
            return r;
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = Vec::new();
    for (_, kind, pts) in &st.objects {
        let shown = if pts.len() > 3000 {
            pts.iter()
                .step_by(pts.len() / 3000 + 1)
                .copied()
                .collect::<Vec<_>>()
        } else {
            pts.clone()
        };
        preview.extend(shown.into_iter().map(|p| Geo::Point { point: p }));
        if !matches!(kind, Editable::Mesh { .. }) {
            if let Some(g) = rebuild(kind, pts) {
                preview.push(g);
            }
            if matches!(kind, Editable::Curve { .. }) {
                preview.push(Geo::Polyline {
                    points: pts.clone(),
                    closed: false,
                });
            }
        }
    }
    let mut r = Reply::ask(
        match st.picked {
            None => format!("{label}: click a point to move (Enter when done)"),
            Some(_) => format!("{label}: click (or type) where the point goes"),
        },
        save(&st),
    )
    .want(Want::Point)
    .preview(preview);
    if let Some((o, i)) = st.picked {
        r.rubber_from = Some(st.objects[o].2[i]);
    }
    r
}

// ---- explode, join -------------------------------------------------------------------------

/// Disconnected pieces of a mesh.
pub fn mesh_parts(m: &SelectedMesh) -> Vec<Mesh> {
    let n = m.vertices.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        let mut i = i;
        while p[i] != r {
            let nx = p[i];
            p[i] = r;
            i = nx;
        }
        r
    }
    // Vertices at the same place belong together (meshes split into faces keep their seams).
    let mut at: BTreeMap<[i64; 3], usize> = BTreeMap::new();
    for (i, v) in m.vertices.iter().enumerate() {
        let k = [
            (v[0] * 1e7).round() as i64,
            (v[1] * 1e7).round() as i64,
            (v[2] * 1e7).round() as i64,
        ];
        if let Some(&j) = at.get(&k) {
            let (a, b) = (find(&mut parent, i), find(&mut parent, j));
            parent[a] = b;
        } else {
            at.insert(k, i);
        }
    }
    for t in &m.triangles {
        let (a, b, c) = (
            find(&mut parent, t[0] as usize),
            find(&mut parent, t[1] as usize),
            find(&mut parent, t[2] as usize),
        );
        parent[a] = c;
        parent[b] = c;
    }
    let mut groups: BTreeMap<usize, Vec<[u32; 3]>> = BTreeMap::new();
    for t in &m.triangles {
        let r = find(&mut parent, t[0] as usize);
        groups.entry(r).or_default().push(*t);
    }
    groups
        .into_values()
        .map(|tris| {
            let mut map = BTreeMap::new();
            let mut verts = Vec::new();
            let tris = tris
                .iter()
                .map(|t| {
                    t.map(|i| {
                        *map.entry(i).or_insert_with(|| {
                            verts.push(m.vertices[i as usize]);
                            (verts.len() - 1) as u32
                        })
                    })
                })
                .collect();
            Mesh {
                vertices: verts,
                triangles: tris,
            }
        })
        .collect()
}

/// Pieces of a curve between its sharp corners (or its segments for a polyline).
pub fn curve_pieces(s: &Selected) -> Vec<Geo> {
    let corners: Option<(Vec<Vec3>, bool)> = match &s.exact {
        Some(Geo::Polyline { points, closed }) => Some((points.clone(), *closed)),
        Some(Geo::Nurbs {
            degree: 1, points, ..
        }) => Some((points.clone(), false)),
        _ => None,
    };
    if let Some((mut pts, closed)) = corners {
        if closed {
            pts.push(pts[0]);
        }
        return pts
            .windows(2)
            .filter(|w| dist(w[0], w[1]) > 1e-12)
            .map(|w| Geo::Line {
                start: w[0],
                end: w[1],
            })
            .collect();
    }
    // Other curves: split at corners sharper than 30°.
    let p = &s.points;
    let mut cuts = vec![0];
    for i in 1..p.len().saturating_sub(1) {
        let (u, v) = (unit(sub(p[i], p[i - 1])), unit(sub(p[i + 1], p[i])));
        if dot(u, v) < 0.866 {
            cuts.push(i);
        }
    }
    cuts.push(p.len() - 1);
    if cuts.len() <= 2 {
        return Vec::new();
    }
    cuts.windows(2)
        .filter_map(|w| interpolate(&p[w[0]..=w[1]], 3, false))
        .collect()
}

fn explode(e: &Event) -> Reply {
    let flow = Flow::new("Explode");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let mut create = Vec::new();
        let mut delete = Vec::new();
        for s in &e.selection {
            let parts: Vec<Geo> = match &s.mesh {
                Some(m) if s.kind != "curve" => {
                    let ps = mesh_parts(m);
                    if ps.len() > 1 {
                        ps.into_iter()
                            .enumerate()
                            .map(|(i, m)| m.geo(&format!("{} part {}", s.name, i + 1), None, None))
                            .collect()
                    } else {
                        Vec::new()
                    }
                }
                _ => curve_pieces(s),
            };
            if parts.len() > 1 {
                create.extend(parts);
                delete.push(s.id.clone());
            }
        }
        if create.is_empty() {
            return Err("nothing to explode (one piece each)".into());
        }
        let n = create.len();
        let mut r =
            Reply::finish(create).say(format!("Explode: {} objects into {n} pieces", delete.len()));
        r.delete = delete;
        Ok(r)
    })
}

fn join(e: &Event) -> Reply {
    let tol = e.number("t11_join_tolerance", 0.01);
    let flow = Flow::new("Join");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let curves: Vec<&Selected> = e
            .selection
            .iter()
            .filter(|s| s.kind == "curve" && s.points.len() >= 2)
            .collect();
        let meshes: Vec<&Selected> = e
            .selection
            .iter()
            .filter(|s| s.kind != "curve" && s.gem.is_none() && s.mesh.is_some())
            .collect();
        let mut create = Vec::new();
        let mut delete = Vec::new();
        if curves.len() >= 2 {
            for (pts, closed) in om9_tools::join_curves(
                &curves.iter().map(|c| c.points.clone()).collect::<Vec<_>>(),
                tol,
            ) {
                create.push(
                    interpolate(&pts, 1, false)
                        .map(|_| Geo::Polyline {
                            points: pts.clone(),
                            closed,
                        })
                        .unwrap(),
                );
            }
            delete.extend(curves.iter().map(|c| c.id.clone()));
        }
        if meshes.len() >= 2 {
            let mut all = Mesh::default();
            for s in &meshes {
                let m = s.mesh.as_ref().unwrap();
                all.append(&Mesh {
                    vertices: m.vertices.clone(),
                    triangles: m.triangles.clone(),
                });
            }
            create.push(om9_addon_geom::shapes::weld(&all, tol * 0.01).geo("Joined", None, None));
            delete.extend(meshes.iter().map(|s| s.id.clone()));
        }
        if create.is_empty() {
            return Err("select two or more curves (or meshes) that meet".into());
        }
        let n = delete.len();
        let mut r = Reply::finish(create).say(format!("Join: {n} objects joined"));
        r.delete = delete;
        Ok(r)
    })
}

// ---- split, trim ---------------------------------------------------------------------------

/// Split a polyline at arc length `s`.
fn split_at(points: &[Vec3], closed: bool, s: f64) -> Vec<Vec<Vec3>> {
    let mut pts = points.to_vec();
    if closed {
        pts.push(pts[0]);
    }
    let path = Path::new(&pts, false).unwrap();
    let cut = path.at(s);
    let mut a: Vec<Vec3> = Vec::new();
    let mut b: Vec<Vec3> = Vec::new();
    let mut acc = 0.0;
    a.push(pts[0]);
    for w in pts.windows(2) {
        let l = dist(w[0], w[1]);
        if acc + l < s {
            a.push(w[1]);
        } else if acc >= s {
            b.push(w[1]);
        } else {
            a.push(cut);
            b.push(cut);
            b.push(w[1]);
        }
        acc += l;
    }
    if closed {
        // One open curve starting and ending at the cut.
        let mut lp = b;
        lp.extend(a.into_iter().skip(1));
        return vec![lp];
    }
    vec![a, b].into_iter().filter(|p| p.len() >= 2).collect()
}

/// A mesh cut in two by the plane through `o` with normal `n` (triangles on the plane split).
pub fn split_mesh(m: &SelectedMesh, o: Vec3, n: Vec3) -> [Mesh; 2] {
    let mut sides = [Mesh::default(), Mesh::default()];
    let d = |p: Vec3| dot(sub(p, o), n);
    for t in &m.triangles {
        let p = t.map(|i| m.vertices[i as usize]);
        let s = p.map(d);
        let mut polys: [Vec<Vec3>; 2] = [Vec::new(), Vec::new()];
        for k in 0..3 {
            let (a, b) = (k, (k + 1) % 3);
            polys[usize::from(s[a] < 0.0)].push(p[a]);
            if (s[a] < 0.0) != (s[b] < 0.0) {
                let x = add(p[a], scale(sub(p[b], p[a]), s[a] / (s[a] - s[b])));
                polys[0].push(x);
                polys[1].push(x);
            }
        }
        for (side, poly) in polys.iter().enumerate() {
            for k in 1..poly.len().saturating_sub(1) {
                let base = sides[side].vertices.len() as u32;
                sides[side].vertices.extend([poly[0], poly[k], poly[k + 1]]);
                sides[side].triangles.push([base, base + 1, base + 2]);
            }
        }
    }
    sides.map(|m| om9_addon_geom::shapes::weld(&m, 1e-9))
}

fn split(e: &Event) -> Reply {
    let flow = Flow::new("Split")
        .point("Point on the curve where to split (meshes: start of the cut line)");
    let has_mesh = e
        .selection
        .iter()
        .any(|s| s.kind != "curve" && s.mesh.is_some());
    let flow = if has_mesh {
        flow.point("End of the cut line")
    } else {
        flow
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let p = c.p(0).unwrap();
        let mut r = Reply::finish(Vec::new());
        let mut n_split = 0;
        for s in &e.selection {
            if s.kind == "curve" && s.points.len() >= 2 {
                let Some(path) = Path::new(&s.points, s.closed) else {
                    continue;
                };
                let at = path.closest(p);
                if dist(path.at(at), p) > 0.05 * path.length().max(1.0) {
                    continue;
                }
                let pieces = split_at(&s.points, s.closed, at);
                let mut geo: Vec<Geo> = pieces
                    .into_iter()
                    .filter_map(|q| {
                        interpolate(
                            &q,
                            if matches!(
                                s.exact,
                                Some(Geo::Polyline { .. }) | Some(Geo::Line { .. })
                            ) {
                                1
                            } else {
                                3
                            },
                            false,
                        )
                    })
                    .collect();
                if geo.is_empty() {
                    continue;
                }
                r.replace.push(Replacement {
                    id: s.id.clone(),
                    geo: geo.remove(0),
                });
                r.create.extend(geo);
                n_split += 1;
            } else if let (Some(m), Some(q)) = (&s.mesh, c.p(1)) {
                let nrm = unit(cross(sub(q, p), e.plane.normal()));
                let [a, b] = split_mesh(m, p, nrm);
                if a.triangles.is_empty() || b.triangles.is_empty() {
                    continue;
                }
                r.replace.push(Replacement {
                    id: s.id.clone(),
                    geo: a.geo(&s.name, None, None),
                });
                r.create
                    .push(b.geo(&format!("{} part", s.name), None, None));
                n_split += 1;
            }
        }
        if n_split == 0 {
            return Err("pick on a selected curve (or a line across a selected mesh)".into());
        }
        Ok(r.say(format!("Split: {n_split} objects")))
    })
}

/// Arc lengths where curve `a` crosses curve `b` (seen along `dir`).
pub fn crossings(a: &[Vec3], b: &[Vec3], dir: Vec3) -> Vec<f64> {
    let d = unit(dir);
    let flat = |p: Vec3| sub(p, scale(d, dot(p, d)));
    let (x, y) = {
        let h = if d[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let x = unit(cross(d, h));
        (x, cross(d, x))
    };
    let two = |p: Vec3| {
        let q = flat(p);
        [dot(q, x), dot(q, y)]
    };
    let mut s = 0.0;
    let mut out = Vec::new();
    for w in a.windows(2) {
        let (p, q) = (two(w[0]), two(w[1]));
        for v in b.windows(2) {
            let (r, t) = (two(v[0]), two(v[1]));
            let den = (q[0] - p[0]) * (t[1] - r[1]) - (q[1] - p[1]) * (t[0] - r[0]);
            if den.abs() < 1e-15 {
                continue;
            }
            let u = ((r[0] - p[0]) * (t[1] - r[1]) - (r[1] - p[1]) * (t[0] - r[0])) / den;
            let w2 = ((r[0] - p[0]) * (q[1] - p[1]) - (r[1] - p[1]) * (q[0] - p[0])) / den;
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&w2) {
                out.push(s + u * dist(w[0], w[1]));
            }
        }
        s += dist(w[0], w[1]);
    }
    out.sort_by(f64::total_cmp);
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    out
}

#[derive(Default, Serialize, Deserialize)]
struct Trimming {
    /// Curves as trimmed so far: (id, pieces).
    curves: Vec<(String, Vec<Vec<Vec3>>)>,
}

fn trim(e: &Event) -> Reply {
    let label = "Trim";
    let mut st: Trimming = load(e);
    if st.curves.is_empty() {
        st.curves = e
            .selection
            .iter()
            .filter(|s| s.kind == "curve" && s.points.len() >= 2)
            .map(|s| {
                let mut p = s.points.clone();
                if s.closed {
                    p.push(p[0]);
                }
                (s.id.clone(), vec![p])
            })
            .collect();
        if st.curves.len() < 2 {
            let r = Reply::ask(
                format!("{label}: select the curves to trim and the cutting curves, Enter"),
                json!({}),
            )
            .want(Want::Objects);
            return if matches!(e.input, Input::Enter) {
                r.refuse(format!("{label}: select two or more crossing curves"))
            } else {
                r
            };
        }
    }
    let dir = e.plane.normal();
    let all: Vec<Vec<Vec3>> = st.curves.iter().flat_map(|c| c.1.clone()).collect();
    let pick = |p: Vec3, st: &Trimming| -> Option<(usize, usize, f64, f64)> {
        // The piece nearest the click and the stretch of it between crossings around the click.
        let d = unit(dir);
        let mut best: Option<(f64, usize, usize, f64)> = None;
        for (ci, (_, pieces)) in st.curves.iter().enumerate() {
            for (pi, piece) in pieces.iter().enumerate() {
                let path = Path::new(piece, false)?;
                let s = path.closest(p);
                let c = sub(path.at(s), p);
                let off = norm(sub(c, scale(d, dot(c, d))));
                if best.is_none_or(|b| off < b.0) {
                    best = Some((off, ci, pi, s));
                }
            }
        }
        let (_, ci, pi, s) = best?;
        let piece = &st.curves[ci].1[pi];
        let len = Path::new(piece, false)?.length();
        let mut cuts: Vec<f64> = all
            .iter()
            .filter(|o| *o != piece)
            .flat_map(|o| crossings(piece, o, dir))
            .filter(|&c| c > 1e-9 && c < len - 1e-9)
            .collect();
        cuts.sort_by(f64::total_cmp);
        let lo = cuts.iter().copied().filter(|&c| c < s).fold(0.0, f64::max);
        let hi = cuts.iter().copied().filter(|&c| c > s).fold(len, f64::min);
        (lo > 0.0 || hi < len).then_some((ci, pi, lo, hi))
    };
    match &e.input {
        Input::Point { point } => {
            let Some((ci, pi, lo, hi)) = pick(*point, &st) else {
                return Reply::ask(
                    format!("{label}: click on a part between crossings"),
                    save(&st),
                )
                .want(Want::Point)
                .refuse(format!("{label}: that part does not cross another curve"));
            };
            let piece = st.curves[ci].1.remove(pi);
            let len = Path::new(&piece, false).map_or(0.0, |p| p.length());
            let mut keep = Vec::new();
            if lo > 1e-9 {
                keep.push(split_at(&piece, false, lo).remove(0));
            }
            if hi < len - 1e-9 {
                keep.push(split_at(&piece, false, hi).pop().unwrap());
            }
            st.curves[ci].1.extend(keep);
        }
        Input::Enter => {
            let mut r = Reply::finish(Vec::new()).say(format!("{label}: done"));
            let originals: BTreeMap<&str, &Selected> =
                e.selection.iter().map(|s| (s.id.as_str(), s)).collect();
            for (id, pieces) in &st.curves {
                let straight = originals.get(id.as_str()).is_some_and(|s| {
                    matches!(s.exact, Some(Geo::Polyline { .. }) | Some(Geo::Line { .. }))
                });
                let geo: Vec<Geo> = pieces
                    .iter()
                    .filter(|p| p.len() >= 2)
                    .filter_map(|p| {
                        if straight {
                            Some(Geo::Polyline {
                                points: p.clone(),
                                closed: false,
                            })
                        } else {
                            interpolate(p, 3, false)
                        }
                    })
                    .collect();
                let unchanged = pieces.len() == 1
                    && originals
                        .get(id.as_str())
                        .is_some_and(|s| pieces[0].len() == s.points.len() + usize::from(s.closed));
                if unchanged {
                    continue;
                }
                let mut it = geo.into_iter();
                match it.next() {
                    Some(g) => r.replace.push(Replacement {
                        id: id.clone(),
                        geo: g,
                    }),
                    None => r.delete.push(id.clone()),
                }
                r.create.extend(it);
            }
            return r;
        }
        _ => {}
    }
    let mut preview: Vec<Geo> = st
        .curves
        .iter()
        .flat_map(|c| {
            c.1.iter().map(|p| Geo::Polyline {
                points: p.clone(),
                closed: false,
            })
        })
        .collect();
    if let Input::Hover { point } = &e.input
        && let Some((ci, pi, lo, hi)) = pick(*point, &st)
    {
        let piece = &st.curves[ci].1[pi];
        if let Some(path) = Path::new(piece, false) {
            let n = 24;
            preview.push(Geo::Polyline {
                points: (0..=n)
                    .map(|k| path.at(lo + (hi - lo) * k as f64 / n as f64))
                    .map(|q| add(q, scale(dir, 0.01)))
                    .collect(),
                closed: false,
            });
        }
    }
    Reply::ask(
        format!("{label}: click the parts to cut away, Enter when done"),
        save(&st),
    )
    .want(Want::Point)
    .preview(preview)
}

// ---- ring rail -----------------------------------------------------------------------------

fn ring_rail(e: &Event) -> Reply {
    let system = e.text("t11_size_system", "us").to_owned();
    let size = e.number("t11_ring_size", 7.0);
    let r = om9_tools::inner_diameter(&system, size) / 2.0;
    let flow = Flow::new("Ring Rail").no_selection();
    flow.step(
        e,
        &|_, _| Show {
            geo: vec![om9_tools::rail_circle(r)],
            ..Default::default()
        },
        &|_| {
            Ok(Reply::finish(vec![om9_tools::rail_circle(r)]).say(format!(
                "Ring Rail: size {} ({}), {} mm inside",
                om9_addon_abi::session::fmt(size),
                system,
                om9_addon_abi::session::fmt(2.0 * r)
            )))
        },
    )
}

/// For tests.
pub fn surface_points(g: &Geo) -> Vec<Vec3> {
    Srf::of(g).map(|s| s.points.to_vec()).unwrap_or_default()
}

#[cfg(test)]
mod tests;
