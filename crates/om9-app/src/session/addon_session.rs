// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive add-on commands (`interactive = true`): Polyline, Circle, Move, Gem Cutter …
//!
//! The controller owns one session at a time. Every input becomes one JSON step to the add-on
//! (see `om9_addon_abi::session`): a click or typed coordinate is a `point`, the cursor moving
//! is a `hover` (preview only, the state does not advance), a clicked or typed command-line
//! option is an `option`, Enter is `enter`. The add-on answers with the prompt, the options
//! ("sub-commands") shown above the command line, a preview drawn over the viewports, and
//! finally the geometry to create or the transform to apply to the selection — committed here
//! in one transaction, so one Undo removes everything the command did.
//!
//! Add-ons work in world millimetres; this module converts to and from document units.
use super::Controller;
use om9_addon_abi::session::{self as s, Event, Geo, Input, Reply, Selected, Want};
use om9_core::{
    Error, Result,
    geometry::{Curve, Geometry, Mesh, NurbsCurve, Point3, Transform},
    model::{Entity, Extension, GEM_EXTENSION_SLOT},
    transaction::{Mutation, TransactionRequest},
};
use serde_json::Value;

/// Samples drawn per previewed curve.
const PREVIEW_SAMPLES: usize = 129;
/// Most selected objects described to an add-on per step.
const MAX_SELECTION: usize = 20_000;
/// Most triangles sent with the selection to a `mesh_data` command per step.
const MAX_SELECTION_TRIANGLES: usize = 2_000_000;

#[derive(Debug, Clone)]
pub struct AddonSession {
    pub code: String,
    pub label: String,
    pub typed: Option<String>,
    pub owner: String,
    /// The last reply that advanced the command (prompt, options, want, preview).
    pub reply: Reply,
    /// Preview lines (document units) of the last reply, the hover preview and the
    /// selection under `preview_transforms`.
    pub lines: Vec<[Point3; 2]>,
    pub hover_lines: Vec<[Point3; 2]>,
    /// Rubber band start (document units) from the last hover or reply.
    pub rubber_from: Option<Point3>,
    /// The last accepted point (document units), for `@dx,dy` input.
    pub last_point: Option<Point3>,
    selection_seen: u64,
    /// The pointer position last sent as a hover event; the UI reports it every frame.
    hover_at: Option<Point3>,
    /// The command reads the triangles of selected meshes (`mesh_data = true`).
    mesh_data: bool,
    /// Every object selected at some step: a command that asks for two groups (Boolean
    /// Difference: first the objects, then the cutters) may change all of them.
    seen: std::collections::BTreeSet<om9_core::identity::ObjectId>,
    /// Where the last selecting click landed (document units): Fillet, Extend, Match … read
    /// which part of a curve was picked.
    pick: Option<Point3>,
}

impl AddonSession {
    /// `Close=No`, `Length`, `Undo` … as shown above the command line, in the interface
    /// language (`Khép kín=Không` in Vietnamese). The first item stays the option's name.
    pub fn tokens(&self, tr: &om9_addons::i18n::Tr) -> Vec<(String, String)> {
        self.reply
            .options
            .iter()
            .map(|o| (o.name.clone(), tr.token(&o.name, o.value.as_deref())))
            .collect()
    }
    pub fn wants(&self) -> Want {
        self.reply.want
    }
}

fn mm(p: Point3, k: f64) -> s::Vec3 {
    [p[0] * k, p[1] * k, p[2] * k]
}
fn doc(p: s::Vec3, k: f64) -> Point3 {
    [p[0] / k, p[1] / k, p[2] / k]
}

/// A row-major add-on matrix (millimetres) as a document transform.
fn to_transform(m: &[f64; 16], k: f64) -> Result<Transform> {
    let mut t = Transform::identity();
    for i in 0..3 {
        for j in 0..3 {
            t.0[i][j] = m[i * 4 + j];
        }
        t.0[i][3] = m[i * 4 + 3] / k;
    }
    if m[12..16] != [0.0, 0.0, 0.0, 1.0] {
        return Err(Error::Invalid("add-on transform"));
    }
    t.validate()?;
    Ok(t)
}

/// The layer a geometry asks for, by name.
fn layer_of(g: &Geo) -> Option<&str> {
    match g {
        Geo::Mesh { layer, .. } => layer.as_deref(),
        _ => None,
    }
}

/// One add-on geometry in document units.
fn to_geometry(g: &Geo, k: f64) -> Result<(Geometry, Option<Value>, String)> {
    let finite = |p: &s::Vec3| p.iter().all(|v| v.is_finite());
    let name = |n: &str| n.to_owned();
    Ok(match g {
        Geo::Point { point } => {
            if !finite(point) {
                return Err(Error::Invalid("add-on point"));
            }
            (Geometry::Point(doc(*point, k)), None, name("Point"))
        }
        Geo::Line { start, end } => (
            Geometry::Curve(Curve::Line {
                start: doc(*start, k),
                end: doc(*end, k),
                domain: [0.0, s::dist(*start, *end) / k],
            }),
            None,
            name("Line"),
        ),
        Geo::Polyline { points, closed } => {
            let mut poles: Vec<Point3> = points.iter().map(|p| doc(*p, k)).collect();
            if *closed && poles.len() > 2 && poles.first() != poles.last() {
                poles.push(poles[0]);
            }
            if poles.len() < 2 {
                return Err(Error::Invalid("polyline needs two points"));
            }
            if poles.len() == 2 {
                return to_geometry(
                    &Geo::Line {
                        start: points[0],
                        end: points[1],
                    },
                    k,
                );
            }
            let n = poles.len();
            let mut knots = vec![0.0; 2];
            knots.extend((1..n - 1).map(|i| i as f64));
            knots.extend([(n - 1) as f64; 2]);
            (
                Geometry::Curve(Curve::Nurbs(NurbsCurve {
                    degree: 1,
                    weights: vec![1.0; n],
                    poles,
                    knots,
                    domain: [0.0, (n - 1) as f64],
                    periodic: false,
                    reversed: false,
                    seam: None,
                })),
                None,
                name("Polyline"),
            )
        }
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => (
            Geometry::Curve(Curve::Circle {
                center: doc(*center, k),
                x: s::unit(*x),
                y: s::unit(*y),
                radius: radius / k,
                domain: [0.0, std::f64::consts::TAU],
                angle_domain: None,
            }),
            None,
            name("Circle"),
        ),
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => {
            if !(*angle > 0.0 && *angle <= std::f64::consts::TAU + 1e-12) {
                return Err(Error::Invalid("arc angle"));
            }
            (
                Geometry::Curve(Curve::Circle {
                    center: doc(*center, k),
                    x: s::unit(*x),
                    y: s::unit(*y),
                    radius: radius / k,
                    domain: [0.0, angle.min(std::f64::consts::TAU)],
                    angle_domain: None,
                }),
                None,
                name("Arc"),
            )
        }
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => {
            let n = points.len();
            let weights = if weights.is_empty() {
                vec![1.0; n]
            } else {
                weights.clone()
            };
            let domain = [
                *knots.get(*degree).ok_or(Error::Invalid("NURBS knots"))?,
                *knots.get(n).ok_or(Error::Invalid("NURBS knots"))?,
            ];
            (
                Geometry::Curve(Curve::Nurbs(NurbsCurve {
                    degree: *degree,
                    poles: points.iter().map(|p| doc(*p, k)).collect(),
                    weights,
                    knots: knots.clone(),
                    domain,
                    periodic: false,
                    reversed: false,
                    seam: None,
                })),
                None,
                name("Curve"),
            )
        }
        Geo::Surface {
            degree,
            size,
            points,
            weights,
            knots,
        } => {
            let n = points.len();
            if n != size[0] * size[1] {
                return Err(Error::Invalid("add-on surface pole grid"));
            }
            let weights = if weights.is_empty() {
                vec![1.0; n]
            } else {
                weights.clone()
            };
            let domain = |i: usize| -> Result<[f64; 2]> {
                Ok([
                    *knots[i].get(degree[i]).ok_or(Error::Invalid("NURBS knots"))?,
                    *knots[i].get(size[i]).ok_or(Error::Invalid("NURBS knots"))?,
                ])
            };
            (
                Geometry::Surface(om9_core::geometry::NurbsSurface {
                    degree: *degree,
                    size: *size,
                    poles: points.iter().map(|p| doc(*p, k)).collect(),
                    weights,
                    knots: knots.clone(),
                    domain: [domain(0)?, domain(1)?],
                    periodic: [false, false],
                    reversed: false,
                }),
                None,
                name("Surface"),
            )
        }
        Geo::Mesh {
            name: n,
            vertices,
            triangles,
            meta,
            ..
        } => {
            if triangles
                .iter()
                .flatten()
                .any(|&i| i as usize >= vertices.len())
            {
                return Err(Error::Invalid("add-on mesh index"));
            }
            (
                Geometry::Mesh(Mesh {
                    vertices: vertices.iter().map(|p| doc(*p, k)).collect(),
                    faces: triangles.iter().map(|t| t.to_vec()).collect(),
                    normals: Vec::new(),
                    colors: Vec::new(),
                    corner_uvs: Vec::new(),
                }),
                meta.clone(),
                if n.is_empty() {
                    "Mesh".into()
                } else {
                    n.clone()
                },
            )
        }
    })
}

/// Line segments (document units) that draw one preview geometry.
fn preview_lines(g: &Geo, k: f64, limits: &om9_core::budget::Limits) -> Vec<[Point3; 2]> {
    let strip =
        |pts: &[Point3]| -> Vec<[Point3; 2]> { pts.windows(2).map(|w| [w[0], w[1]]).collect() };
    match g {
        Geo::Point { point } => {
            let p = doc(*point, k);
            let d = 0.3 / k;
            vec![
                [[p[0] - d, p[1], p[2]], [p[0] + d, p[1], p[2]]],
                [[p[0], p[1] - d, p[2]], [p[0], p[1] + d, p[2]]],
            ]
        }
        Geo::Mesh {
            vertices,
            triangles,
            ..
        } => super::addon_builder::feature_edges(vertices, triangles)
            .into_iter()
            .map(|[a, b]| [doc(vertices[a], k), doc(vertices[b], k)])
            .collect(),
        Geo::Polyline { points, closed } => {
            let mut pts: Vec<Point3> = points.iter().map(|p| doc(*p, k)).collect();
            if *closed && pts.len() > 2 {
                pts.push(pts[0]);
            }
            strip(&pts)
        }
        other => match to_geometry(other, k) {
            Ok((Geometry::Curve(c), _, _)) => {
                om9_core::surface_kernel::sample_curve(&c, PREVIEW_SAMPLES, limits)
                    .map(|p| strip(&p))
                    .unwrap_or_default()
            }
            // A surface previews as a grid of isocurves.
            Ok((Geometry::Surface(srf), _, _)) => {
                let mut out = Vec::new();
                let n = 12;
                let (du, dv) = (srf.domain[0], srf.domain[1]);
                for dir in 0..2 {
                    for i in 0..=n {
                        let fixed = i as f64 / n as f64;
                        let pts: Vec<Point3> = (0..=32)
                            .filter_map(|j| {
                                let t = j as f64 / 32.0;
                                let (a, b) = if dir == 0 { (fixed, t) } else { (t, fixed) };
                                om9_core::surface_kernel::evaluate(
                                    &srf,
                                    du[0] + a * (du[1] - du[0]),
                                    dv[0] + b * (dv[1] - dv[0]),
                                    limits,
                                )
                                .ok()
                            })
                            .collect();
                        out.extend(strip(&pts));
                    }
                }
                out
            }
            _ => Vec::new(),
        },
    }
}

/// A gem record's frame (`center`, `axis`, `x`, mm at creation) moved by the object's
/// placement, so a moved or rotated gem still reports where it really is.
fn gem_in_world(mut g: Value, placement: &Transform, k: f64) -> Value {
    let read = |v: &Value| -> Option<[f64; 3]> {
        let a = v.as_array()?;
        Some([
            a.first()?.as_f64()?,
            a.get(1)?.as_f64()?,
            a.get(2)?.as_f64()?,
        ])
    };
    if let Some(c) = g.get("center").and_then(read)
        && let Ok(w) = placement.point(doc(c, k))
    {
        g["center"] = serde_json::json!(mm(w, k));
    }
    for key in ["axis", "x"] {
        if let Some(d) = g.get(key).and_then(read) {
            let m = placement.0;
            let v: [f64; 3] = std::array::from_fn(|i| (0..3).map(|j| m[i][j] * d[j]).sum());
            g[key] = serde_json::json!(s::unit(v));
        }
    }
    // Uniform scale changes the stone size too.
    let m = placement.0;
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    for key in ["diameter_mm", "length_mm", "width_mm", "depth_mm"] {
        if let Some(v) = g.get(key).and_then(Value::as_f64) {
            g[key] = serde_json::json!(v * det.abs().cbrt());
        }
    }
    g
}

/// Inverse of an object placement (3×4, row-major), for world → local.
fn inverse(placement: &Transform) -> Result<[[f64; 4]; 3]> {
    let m = placement.0;
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-12 {
        return Err(Error::Invalid("object placement"));
    }
    let c = |r: usize, col: usize| {
        let (r1, r2) = ((r + 1) % 3, (r + 2) % 3);
        let (c1, c2) = ((col + 1) % 3, (col + 2) % 3);
        m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1]
    };
    let mut inv = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            inv[i][j] = c(j, i) / det;
        }
    }
    for i in 0..3 {
        inv[i][3] = -(0..3).map(|j| inv[i][j] * m[j][3]).sum::<f64>();
    }
    Ok(inv)
}

/// World add-on geometry (mm) in an object's local frame (still mm).
fn geo_in_local(g: &Geo, placement: &Transform, k: f64) -> Result<Geo> {
    let inv = inverse(placement)?;
    let p = |v: s::Vec3| -> s::Vec3 {
        let d = doc(v, k);
        let l: [f64; 3] = std::array::from_fn(|i| {
            inv[i][0] * d[0] + inv[i][1] * d[1] + inv[i][2] * d[2] + inv[i][3]
        });
        mm(l, k)
    };
    let v = |d: s::Vec3| -> s::Vec3 {
        std::array::from_fn(|i| inv[i][0] * d[0] + inv[i][1] * d[1] + inv[i][2] * d[2])
    };
    let sc = s::norm(v([1.0, 0.0, 0.0]));
    Ok(match g {
        Geo::Point { point } => Geo::Point { point: p(*point) },
        Geo::Line { start, end } => Geo::Line {
            start: p(*start),
            end: p(*end),
        },
        Geo::Polyline { points, closed } => Geo::Polyline {
            points: points.iter().map(|q| p(*q)).collect(),
            closed: *closed,
        },
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => Geo::Circle {
            center: p(*center),
            x: s::unit(v(*x)),
            y: s::unit(v(*y)),
            radius: radius * sc,
        },
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => Geo::Arc {
            center: p(*center),
            x: s::unit(v(*x)),
            y: s::unit(v(*y)),
            radius: radius * sc,
            angle: *angle,
        },
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => Geo::Nurbs {
            degree: *degree,
            points: points.iter().map(|q| p(*q)).collect(),
            weights: weights.clone(),
            knots: knots.clone(),
        },
        Geo::Mesh {
            name,
            vertices,
            triangles,
            meta,
            layer,
        } => Geo::Mesh {
            name: name.clone(),
            vertices: vertices.iter().map(|q| p(*q)).collect(),
            triangles: triangles.clone(),
            meta: meta.clone(),
            layer: layer.clone(),
        },
        Geo::Surface {
            degree,
            size,
            points,
            weights,
            knots,
        } => Geo::Surface {
            degree: *degree,
            size: *size,
            points: points.iter().map(|q| p(*q)).collect(),
            weights: weights.clone(),
            knots: knots.clone(),
        },
        other => other.clone(),
    })
}

/// The exact definition of a curve or NURBS surface object, in world mm.
fn exact_in_world(g: &Geometry, placement: &Transform, k: f64) -> Option<Geo> {
    let p = |q: Point3| placement.point(q).ok().map(|w| mm(w, k));
    let m = placement.0;
    let v = |d: Point3| -> s::Vec3 { std::array::from_fn(|i| (0..3).map(|j| m[i][j] * d[j]).sum()) };
    let sc = s::norm(v([1.0, 0.0, 0.0]));
    Some(match g {
        Geometry::Curve(Curve::Line { start, end, .. }) => Geo::Line {
            start: p(*start)?,
            end: p(*end)?,
        },
        Geometry::Curve(Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        }) => {
            let span = angle_domain.unwrap_or(*domain);
            let angle = span[1] - span[0];
            let (c, xs, ys) = (p(*center)?, s::unit(v(*x)), s::unit(v(*y)));
            let r = radius * sc * k;
            if angle >= std::f64::consts::TAU - 1e-9 {
                Geo::Circle {
                    center: c,
                    x: xs,
                    y: ys,
                    radius: r,
                }
            } else {
                // Start the arc at its first angle.
                let (sa, ca) = span[0].sin_cos();
                let x0 = s::add(s::scale(xs, ca), s::scale(ys, sa));
                let y0 = s::cross(s::unit(s::cross(xs, ys)), x0);
                Geo::Arc {
                    center: c,
                    x: x0,
                    y: y0,
                    radius: r,
                    angle,
                }
            }
        }
        Geometry::Curve(Curve::Nurbs(n)) => Geo::Nurbs {
            degree: n.degree,
            points: n.poles.iter().map(|q| p(*q)).collect::<Option<_>>()?,
            weights: n.weights.clone(),
            knots: n.knots.clone(),
        },
        Geometry::Surface(srf) => Geo::Surface {
            degree: srf.degree,
            size: srf.size,
            points: srf.poles.iter().map(|q| p(*q)).collect::<Option<_>>()?,
            weights: srf.weights.clone(),
            knots: srf.knots.clone(),
        },
        _ => return None,
    })
}

/// A world gem record back into the object's own frame (inverse of [`gem_in_world`]).
fn record_in_local(mut g: Value, placement: &Transform, k: f64) -> Result<Value> {
    let m = placement.0;
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-12 {
        return Err(Error::Invalid("object placement"));
    }
    // Inverse of the 3×3 part by cofactors.
    let c = |r: usize, col: usize| {
        let (r1, r2) = ((r + 1) % 3, (r + 2) % 3);
        let (c1, c2) = ((col + 1) % 3, (col + 2) % 3);
        m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1]
    };
    let inv: [[f64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| c(j, i) / det));
    let mul = |v: [f64; 3]| -> [f64; 3] {
        std::array::from_fn(|i| (0..3).map(|j| inv[i][j] * v[j]).sum())
    };
    let read = |v: &Value| -> Option<[f64; 3]> {
        let a = v.as_array()?;
        Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?])
    };
    if let Some(cw) = g.get("center").and_then(read) {
        let w = doc(cw, k);
        let d = [w[0] - m[0][3], w[1] - m[1][3], w[2] - m[2][3]];
        g["center"] = serde_json::json!(mm(mul(d), k));
    }
    for key in ["axis", "x"] {
        if let Some(d) = g.get(key).and_then(read) {
            g[key] = serde_json::json!(s::unit(mul(d)));
        }
    }
    let scale = det.abs().cbrt();
    for key in ["diameter_mm", "length_mm", "width_mm", "depth_mm"] {
        if let Some(v) = g.get(key).and_then(Value::as_f64) {
            g[key] = serde_json::json!(v / scale);
        }
    }
    Ok(g)
}

impl Controller {
    pub fn addon_session(&self) -> Option<&AddonSession> {
        self.addon_session.as_ref()
    }

    /// Whether the open session is waiting for a viewport click.
    pub fn session_wants_point(&self) -> bool {
        self.addon_session
            .as_ref()
            .is_some_and(|s| matches!(s.reply.want, Want::Point | Want::Number))
    }

    /// The selection as add-ons see it (world millimetres).
    fn session_selection(&self, k: f64, mesh_data: bool) -> Vec<Selected> {
        use om9_scene::scene::RenderBuffers;
        let mut out: Vec<Selected> = Vec::new();
        let mut triangles_sent = 0usize;
        // Nothing picked: nothing to walk (the scene can hold a whole large file).
        if self.selection.is_empty() {
            return out;
        }
        let Some(scene) = self.scene.as_ref() else {
            return out;
        };
        let mut index = std::collections::BTreeMap::new();
        for object in &scene.objects {
            if !self.selection.contains(&object.root) || out.len() >= MAX_SELECTION {
                continue;
            }
            let Some(entity) = self.document().entities.get(&object.root) else {
                continue;
            };
            let slot = *index.entry(object.root).or_insert_with(|| {
                let kind = match &entity.geometry {
                    Geometry::Curve(_) => "curve",
                    Geometry::Mesh(_) => "mesh",
                    Geometry::Point(_) => "point",
                    Geometry::Brep(_) => "brep",
                    Geometry::Surface(_) => "surface",
                    _ => "other",
                };
                let gem = entity
                    .extension(GEM_EXTENSION_SLOT)
                    .and_then(|x| serde_json::from_str::<Value>(&x.payload).ok())
                    .map(|g| gem_in_world(g, &entity.placement, k));
                out.push(Selected {
                    id: object.root.to_string(),
                    kind: kind.into(),
                    name: entity.label.clone(),
                    bbox: [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]],
                    points: Vec::new(),
                    closed: false,
                    gem,
                    mesh: None,
                    exact: exact_in_world(&entity.geometry, &entity.placement, k),
                });
                out.len() - 1
            });
            let Ok(t) = object.display_transform() else {
                continue;
            };
            let (points, is_curve) = match object.buffers.as_ref() {
                RenderBuffers::Curve { points, .. } => (points, true),
                RenderBuffers::Points { points, .. } => (points, false),
                RenderBuffers::Mesh { vertices, .. } => (vertices, false),
            };
            let sel = &mut out[slot];
            if mesh_data
                && let RenderBuffers::Mesh { triangles, .. } = object.buffers.as_ref()
                && triangles_sent + triangles.len() <= MAX_SELECTION_TRIANGLES
            {
                triangles_sent += triangles.len();
                let mesh = sel.mesh.get_or_insert_with(Default::default);
                let base = mesh.vertices.len() as u32;
                mesh.vertices.extend(
                    points
                        .iter()
                        .map(|p| t.point(*p).map(|w| mm(w, k)).unwrap_or(mm(*p, k))),
                );
                mesh.triangles.extend(
                    triangles
                        .iter()
                        .map(|(tri, _)| [tri[0] + base, tri[1] + base, tri[2] + base]),
                );
            }
            for p in points {
                let Ok(w) = t.point(*p) else { continue };
                let w = mm(w, k);
                for i in 0..3 {
                    sel.bbox[0][i] = sel.bbox[0][i].min(w[i]);
                    sel.bbox[1][i] = sel.bbox[1][i].max(w[i]);
                }
                if is_curve && sel.points.len() < 4096 {
                    sel.points.push(w);
                }
            }
        }
        for sel in &mut out {
            if sel.kind == "curve" && sel.points.len() > 2 {
                let (a, b) = (sel.points[0], sel.points[sel.points.len() - 1]);
                if s::dist(a, b) <= 1e-6 {
                    sel.closed = true;
                    sel.points.pop();
                }
            }
        }
        out
    }

    /// Lines (document units) of the selection under each transform, for Move/Rotate previews.
    fn transformed_selection_lines(&self, matrices: &[[f64; 16]], k: f64) -> Vec<[Point3; 2]> {
        use om9_scene::scene::RenderBuffers;
        let mut lines = Vec::new();
        // Most commands preview no transform: skip the walk over the scene.
        if matrices.is_empty() || self.selection.is_empty() {
            return lines;
        }
        let Some(scene) = self.scene.as_ref() else {
            return lines;
        };
        let transforms: Vec<Transform> = matrices
            .iter()
            .filter_map(|m| to_transform(m, k).ok())
            .collect();
        for object in &scene.objects {
            if !self.selection.contains(&object.root) || lines.len() > 200_000 {
                continue;
            }
            let Ok(display) = object.display_transform() else {
                continue;
            };
            let world: Vec<Point3> = match object.buffers.as_ref() {
                RenderBuffers::Curve { points, .. } => points
                    .iter()
                    .filter_map(|p| display.point(*p).ok())
                    .collect(),
                RenderBuffers::Points { points, .. }
                | RenderBuffers::Mesh {
                    vertices: points, ..
                } => {
                    // Bounding box edges stand for a mesh or a point cloud.
                    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
                    for p in points {
                        if let Ok(w) = display.point(*p) {
                            for i in 0..3 {
                                lo[i] = lo[i].min(w[i]);
                                hi[i] = hi[i].max(w[i]);
                            }
                        }
                    }
                    if !lo[0].is_finite() {
                        continue;
                    }
                    let c = |i: usize| -> Point3 {
                        [
                            if i & 1 == 0 { lo[0] } else { hi[0] },
                            if i & 2 == 0 { lo[1] } else { hi[1] },
                            if i & 4 == 0 { lo[2] } else { hi[2] },
                        ]
                    };
                    for t in &transforms {
                        for (a, b) in [
                            (0, 1),
                            (1, 3),
                            (3, 2),
                            (2, 0),
                            (4, 5),
                            (5, 7),
                            (7, 6),
                            (6, 4),
                            (0, 4),
                            (1, 5),
                            (2, 6),
                            (3, 7),
                        ] {
                            if let (Ok(p), Ok(q)) = (t.point(c(a)), t.point(c(b))) {
                                lines.push([p, q]);
                            }
                        }
                    }
                    continue;
                }
            };
            for t in &transforms {
                let pts: Vec<Point3> = world.iter().filter_map(|p| t.point(*p).ok()).collect();
                lines.extend(pts.windows(2).map(|w| [w[0], w[1]]));
            }
        }
        lines
    }

    /// Start an interactive add-on command.
    pub(super) fn start_addon_session(&mut self, code: &str, label: &str) -> Result<()> {
        let owner = self
            .addons
            .as_ref()
            .and_then(|h| {
                h.lock()
                    .ok()
                    .and_then(|h| h.owner_of(code).map(str::to_owned))
            })
            .ok_or(Error::Unsupported("add-on command"))?;
        let mesh_data = self
            .addons
            .as_ref()
            .and_then(|h| h.lock().ok()?.command_info(code))
            .is_some_and(|i| i.command.mesh_data);
        self.cancel();
        self.addon_session = Some(AddonSession {
            code: code.to_owned(),
            label: label.to_owned(),
            typed: self.addon_typed.get(code).cloned(),
            owner,
            reply: Reply::default(),
            lines: Vec::new(),
            hover_lines: Vec::new(),
            rubber_from: None,
            last_point: None,
            selection_seen: self.selection_revision,
            hover_at: None,
            mesh_data,
            seen: Default::default(),
            pick: None,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            code,
            "interactive",
            None,
        );
        if let Err(e) = self.session_step(Input::Start) {
            self.addon_session = None;
            return Err(e);
        }
        Ok(())
    }

    /// Send one input; adopt the reply (or, for `hover`, only its preview).
    pub(super) fn session_step(&mut self, input: Input) -> Result<()> {
        let Some(session) = self.addon_session.as_ref() else {
            return Err(Error::Missing("interactive command"));
        };
        let k = self.document().page_unit.factor();
        let hover = matches!(input, Input::Hover { .. });
        let plane = self.input_plane();
        let settings = self
            .addon_builder
            .as_ref()
            .filter(|b| b.code == session.code)
            .map(|b| b.values.clone().into_iter().collect::<Vec<_>>())
            .or_else(|| {
                let host = self.addons.as_ref()?.lock().ok()?;
                // Only this command's own parameters (its builder panel).
                Some(host.command_settings(&session.owner, &session.code))
            })
            .unwrap_or_default()
            .into_iter()
            .map(|(key, v)| {
                let json = match v {
                    om9_addons::settings::SettingValue::Bool(b) => Value::Bool(b),
                    om9_addons::settings::SettingValue::Int(i) => Value::from(i),
                    om9_addons::settings::SettingValue::Float(f) => Value::from(f),
                    om9_addons::settings::SettingValue::Str(t) => Value::String(t),
                };
                (key, json)
            })
            .collect();
        let current: Vec<_> = self.selection.iter().copied().collect();
        if let Some(s) = self.addon_session.as_mut() {
            s.seen.extend(current);
        }
        let Some(session) = self.addon_session.as_ref() else {
            return Err(Error::Missing("interactive command"));
        };
        let event = Event {
            command: session.code.clone(),
            input,
            state: session.reply.state.clone(),
            plane: s::Plane {
                origin: mm(plane.origin, k),
                x: plane.x,
                y: plane.y,
            },
            selection: self.session_selection(k, session.mesh_data),
            pick: session.pick.map(|p| mm(p, k)),
            settings,
        };
        let bytes = serde_json::to_vec(&event).map_err(|e| Error::Io(e.to_string()))?;
        let host = self
            .addons
            .clone()
            .ok_or(Error::Unsupported("add-on host"))?;
        let answer = host
            .lock()
            .map_err(|_| Error::Io("add-on host poisoned".into()))?
            .session(&event.command, &bytes);
        let reply: Reply = match answer {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| Error::Io(format!("{}: trả lời không hợp lệ: {e}", event.command)))?,
            Err(e) => {
                if !hover {
                    self.cancel_addon_session();
                }
                return Err(Error::Io(e));
            }
        };
        let limits = *self.coordinator.limits();
        if hover {
            let mut lines: Vec<[Point3; 2]> = reply
                .preview
                .iter()
                .flat_map(|g| preview_lines(g, k, &limits))
                .collect();
            lines.extend(self.transformed_selection_lines(&reply.preview_transforms, k));
            let rubber = reply.rubber_from.map(|p| doc(p, k));
            if let Some(session) = self.addon_session.as_mut() {
                session.hover_lines = lines;
                session.rubber_from = rubber.or(session.rubber_from);
                // A command that follows the pointer (Sketch) keeps what the move added.
                if reply.track && reply.error.is_none() && !reply.done {
                    session.reply.state = reply.state.clone();
                }
            }
            return Ok(());
        }
        let tr = self.tr.clone();
        self.addon_messages
            .extend(reply.messages.iter().map(|m| tr.text(m)));
        if reply.done {
            let (code, label, seen) = {
                let s = self.addon_session.as_ref().expect("session");
                (s.code.clone(), s.label.clone(), s.seen.clone())
            };
            self.addon_session = None;
            let created = self.commit_session_result(&code, &label, &reply, &seen)?;
            // The command's panel closes with it, however it was finished (Enter in the
            // command line, right-click, Accept); its values become the next defaults.
            if let Some(b) = self
                .addon_builder
                .take_if(|b| b.interactive && b.code == code)
            {
                self.store_builder_values(&b);
            }
            self.events.emit(
                crate::command_events::EventKind::Finished,
                &code,
                &format!("created {created} objects"),
                Some(crate::command_events::EventOutcome::Committed),
            );
            return Ok(());
        }
        let mut lines: Vec<[Point3; 2]> = reply
            .preview
            .iter()
            .flat_map(|g| preview_lines(g, k, &limits))
            .collect();
        lines.extend(self.transformed_selection_lines(&reply.preview_transforms, k));
        let error = reply.error.clone();
        if let Some(session) = self.addon_session.as_mut() {
            session.rubber_from = reply.rubber_from.map(|p| doc(p, k));
            session.hover_lines.clear();
            session.lines = lines;
            session.reply = reply;
        }
        match error {
            Some(e) => Err(Error::Io(tr.text(&e))),
            None => Ok(()),
        }
    }

    /// A viewport click while the session waits for a point.
    pub fn session_point(&mut self, point: Point3) -> Result<()> {
        let k = self.document().page_unit.factor();
        let result = self.session_step(Input::Point {
            point: mm(point, k),
        });
        if result.is_ok()
            && let Some(s) = self.addon_session.as_mut()
        {
            s.last_point = Some(point);
        }
        result
    }

    /// The cursor moved over a viewport.
    pub fn session_hover(&mut self, point: Point3) {
        if let Some(s) = self.addon_session.as_mut()
            && matches!(s.reply.want, Want::Point | Want::Number)
            && s.hover_at != Some(point)
        {
            s.hover_at = Some(point);
            let k = self.document().page_unit.factor();
            let _ = self.session_step(Input::Hover {
                point: mm(point, k),
            });
        }
    }

    /// A click in a viewport that may select an object while a session waits for objects:
    /// remember where it landed, so the command knows which part of the curve was picked.
    pub fn note_session_pick(&mut self, point: Option<Point3>) {
        if let Some(s) = self.addon_session.as_mut()
            && matches!(s.reply.want, Want::Objects)
        {
            s.pick = point;
        }
    }

    /// Called every frame: a session waiting for objects hears about selection changes.
    pub fn refresh_addon_session(&mut self) {
        let Some(s) = self.addon_session.as_ref() else {
            return;
        };
        if s.selection_seen != self.selection_revision {
            let rev = self.selection_revision;
            if let Some(s) = self.addon_session.as_mut() {
                s.selection_seen = rev;
            }
            let _ = self.session_step(Input::Selection);
        }
    }

    /// A line typed at the command line while a session is open. `Ok(false)`: not for the
    /// session (another command name) — the session closed and the caller runs the text.
    pub(super) fn submit_session_text(&mut self, text: &str) -> Result<bool> {
        let Some(session) = self.addon_session.as_ref() else {
            return Ok(false);
        };
        let t = text.trim();
        if t.is_empty() {
            self.session_step(Input::Enter)?;
            return Ok(true);
        }
        // A command with a builder panel: its options (`GirdleScale=110`) set panel values.
        if let Some(b) = self
            .addon_builder
            .as_ref()
            .filter(|b| b.code == session.code)
        {
            let (name, value) = match t.split_once('=') {
                Some((n, v)) => (n.trim(), Some(v.trim())),
                None => (t, None),
            };
            if let Some(spec) = b.find(name) {
                let key = spec.key.clone();
                let steppable = matches!(
                    spec.kind,
                    om9_addons::settings::Kind::Bool | om9_addons::settings::Kind::Enum { .. }
                );
                match value {
                    Some(v) => self.set_builder_text(&key, v)?,
                    None if steppable => self.step_builder_value(&key)?,
                    None => {
                        return Err(Error::Io(format!(
                            "{}: gõ {}=giá trị",
                            b.label, spec.option
                        )));
                    }
                }
                self.refresh_builder_preview();
                return Ok(true);
            }
        }
        let (name, value) = match t.split_once('=') {
            Some((n, v)) => (n.trim(), Some(v.trim().to_owned())),
            None => (t, None),
        };
        // The English name always works; in Vietnamese the label works too, with or without
        // accents and spaces (`BothSides`, `Hai phía`, `haiphia`).
        let tr = &self.tr;
        let folded = om9_addons::i18n::fold(name);
        let options = &session.reply.options;
        let exact = options.iter().find(|o| tr.names(name, &o.name));
        let option = exact.or_else(|| {
            let mut hits = options.iter().filter(|o| tr.starts(name, &o.name));
            match (hits.next(), hits.next()) {
                (Some(o), None) if !folded.is_empty() && value.is_some() || folded.len() >= 2 => {
                    Some(o)
                }
                _ => None,
            }
        });
        if let Some(o) = option {
            let name = o.name.clone();
            let value = value.map(|v| tr.english_value(&v));
            if name.eq_ignore_ascii_case("undo") && value.is_none() {
                self.session_step(Input::Undo)?;
            } else {
                self.session_step(Input::Option { name, value })?;
            }
            return Ok(true);
        }
        // Coordinates and numbers belong to the session; another command name closes it.
        let k = self.document().page_unit.factor();
        let plane = self.input_plane();
        let last = session.last_point.map(|p| mm(p, k));
        let splane = s::Plane {
            origin: mm(plane.origin, k),
            x: plane.x,
            y: plane.y,
        };
        let is_input = s::parse_point(t, &splane, last).is_some() || s::parse_number(t).is_some();
        let is_command =
            !is_input && (Self::resolve_alias(t).is_some() || self.addon_code(t).is_some());
        if is_command {
            self.cancel_addon_session();
            return Ok(false);
        }
        if let Some(p) = s::parse_point(t, &splane, last)
            && matches!(session.reply.want, Want::Point)
        {
            let point = doc(p, k);
            self.session_step(Input::Point { point: p })?;
            if let Some(s) = self.addon_session.as_mut() {
                s.last_point = Some(point);
            }
            return Ok(true);
        }
        self.session_step(Input::Text { text: t.to_owned() })?;
        Ok(true)
    }

    /// Esc: close the session; nothing is created.
    pub fn cancel_addon_session(&mut self) -> bool {
        match self.addon_session.take() {
            Some(s) => {
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    &s.code,
                    "cancelled",
                    Some(crate::command_events::EventOutcome::Cancelled),
                );
                true
            }
            None => false,
        }
    }

    pub(super) fn session_prompt(&self) -> Option<String> {
        let s = self.addon_session.as_ref()?;
        let p = s.reply.prompt.trim();
        Some(if p.is_empty() {
            format!("{}: ", self.tr.command(&s.code, &s.label))
        } else {
            format!("{}: ", self.tr.text(p))
        })
    }

    /// Create the geometry and apply the transform of a finished session, in one transaction.
    fn commit_session_result(
        &mut self,
        code: &str,
        label: &str,
        reply: &Reply,
        seen: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    ) -> Result<usize> {
        let k = self.document().page_unit.factor();
        let layer = self.document().current_layer;
        let mut mutations = Vec::new();
        let mut created_ids = Vec::new();
        for g in &reply.create {
            let (geometry, meta, name) = to_geometry(g, k)?;
            let layer = layer_of(g)
                .and_then(|wanted| {
                    self.document()
                        .layers
                        .iter()
                        .find(|(_, l)| l.name.eq_ignore_ascii_case(wanted))
                        .map(|(id, _)| *id)
                })
                .or(layer);
            let mut entity = Entity::new(&name, geometry, layer);
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            if let Some(meta) = meta
                && let Ok(x) = Extension::new(1, meta.to_string())
            {
                entity.extensions.insert(GEM_EXTENSION_SLOT.to_owned(), x);
            }
            created_ids.push(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        let selected = |id: &str| -> Result<om9_core::identity::ObjectId> {
            self.selection
                .iter()
                .chain(seen.iter())
                .copied()
                .find(|o| o.to_string() == id)
                .filter(|o| self.document().entities.contains_key(o))
                .ok_or(Error::Invalid("add-on result names an object that is not selected"))
        };
        if let Some(op) = reply.transform.as_ref().filter(|op| !op.per_object.is_empty()) {
            for item in &op.per_object {
                let id = selected(&item.id)?;
                let t = to_transform(&item.matrix, k)?;
                let entity = self
                    .document()
                    .entities
                    .get(&id)
                    .ok_or(Error::Missing("object"))?
                    .clone();
                if op.keep_original {
                    let mut copy =
                        Entity::new(&entity.label, entity.geometry.clone(), entity.attributes.layer);
                    copy.attributes = entity.attributes.clone();
                    copy.extensions = entity.extensions.clone();
                    copy.placement = t.compose(entity.placement)?;
                    created_ids.push(copy.id);
                    mutations.push(Mutation::Insert {
                        document: self.document,
                        entity: copy,
                    });
                } else {
                    mutations.push(Mutation::Placement {
                        source: self.document().stamp(id)?,
                        placement: t.compose(entity.placement)?,
                    });
                }
            }
        }
        for r in &reply.replace {
            let id = selected(&r.id)?;
            let placement = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("object"))?
                .placement;
            // The object keeps its placement: the world geometry goes into its own frame.
            let local = geo_in_local(&r.geo, &placement, k)?;
            let (geometry, _, _) = to_geometry(&local, k)?;
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(id)?,
                geometry,
            });
        }
        for u in &reply.records {
            let id = selected(&u.id)?;
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("object"))?;
            let local = record_in_local(u.record.clone(), &entity.placement, k)?;
            let mut extensions = entity.extensions.clone();
            extensions.insert(
                GEM_EXTENSION_SLOT.to_owned(),
                Extension::new(1, local.to_string())?,
            );
            mutations.push(Mutation::Extensions {
                source: self.document().stamp(id)?,
                extensions,
            });
        }
        for id in &reply.delete {
            let id = selected(id)?;
            mutations.push(Mutation::Delete {
                source: self.document().stamp(id)?,
            });
        }
        if let Some(op) = reply.transform.as_ref().filter(|op| op.per_object.is_empty()) {
            let transforms: Vec<Transform> = op
                .matrices
                .iter()
                .map(|m| to_transform(m, k))
                .collect::<Result<_>>()?;
            let targets: Vec<_> = self.selection.iter().copied().collect();
            if targets.is_empty() {
                return Err(Error::Missing("selection"));
            }
            for id in &targets {
                let entity = self
                    .document()
                    .entities
                    .get(id)
                    .ok_or(Error::Missing("object"))?
                    .clone();
                if !op.keep_original && transforms.len() == 1 {
                    mutations.push(Mutation::Placement {
                        source: self.document().stamp(*id)?,
                        placement: transforms[0].compose(entity.placement)?,
                    });
                } else {
                    for t in &transforms {
                        let mut copy = Entity::new(
                            &entity.label,
                            entity.geometry.clone(),
                            entity.attributes.layer,
                        );
                        copy.attributes = entity.attributes.clone();
                        copy.extensions = entity.extensions.clone();
                        copy.placement = t.compose(entity.placement)?;
                        created_ids.push(copy.id);
                        mutations.push(Mutation::Insert {
                            document: self.document,
                            entity: copy,
                        });
                    }
                }
            }
        }
        if mutations.is_empty() {
            return Ok(0);
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("command scene")); }
        let count = mutations.len();
        let mut held = self.scene_cache_usage()?;
        let undo_label = crate::icons::icon_name(code).unwrap_or(label).to_owned();
        // One new object (Line, Circle …): add it to the scene incrementally, as OM9's own
        // commands do, instead of rebuilding the scene of the whole document.
        if let [Mutation::Insert { entity, .. }] = mutations.as_slice() {
            let increment = scene.update_held_usage(Some(entity))?;
            held.add(increment, self.coordinator.limits())?;
            let added = entity.id;
            let prepared = self
                .coordinator
                .prepare_with_held_usage(TransactionRequest::new(&undo_label, mutations), held)?;
            let receipt = self.commit_incremental_scene(prepared, held, increment, Some(added), false)?;
            if receipt.published {
                self.sync_published(om9_core::changes::ChangeCause::Edit);
            }
            return Ok(count);
        }
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(&undo_label, mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_on_geometry_converts_to_document_curves() {
        let l = om9_core::budget::Limits::default();
        let (g, _, n) = to_geometry(
            &Geo::Polyline {
                points: vec![[0.0; 3], [10.0, 0.0, 0.0], [10.0, 5.0, 0.0]],
                closed: true,
            },
            1.0,
        )
        .unwrap();
        assert_eq!(n, "Polyline");
        let Geometry::Curve(c) = &g else { panic!() };
        c.validate(&l).unwrap();
        let pts = om9_core::surface_kernel::sample_curve(c, 5, &l).unwrap();
        assert_eq!(pts[0], pts[4], "closed polyline returns to its start");
        // Document in centimetres: 10 mm → 1 cm.
        let (g, _, _) = to_geometry(
            &Geo::Circle {
                center: [10.0, 0.0, 0.0],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius: 5.0,
            },
            10.0,
        )
        .unwrap();
        let Geometry::Curve(Curve::Circle { center, radius, .. }) = g else {
            panic!()
        };
        assert_eq!((center, radius), ([1.0, 0.0, 0.0], 0.5));
        let arc = Geo::Arc {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            angle: std::f64::consts::FRAC_PI_2,
        };
        let (Geometry::Curve(c), _, _) = to_geometry(&arc, 1.0).unwrap() else {
            panic!()
        };
        let pts = om9_core::surface_kernel::sample_curve(&c, 3, &l).unwrap();
        assert!(
            (pts[2][0]).abs() < 1e-9 && (pts[2][1] - 2.0).abs() < 1e-9,
            "{pts:?}"
        );
        assert!(
            to_geometry(
                &Geo::Polyline {
                    points: vec![[0.0; 3]],
                    closed: false
                },
                1.0
            )
            .is_err()
        );
        assert!(!preview_lines(&arc, 1.0, &l).is_empty());
    }

    #[test]
    fn add_on_matrices_scale_their_translation_to_document_units() {
        let m = s::translation([10.0, 0.0, 0.0]);
        let t = to_transform(&m, 10.0).unwrap();
        assert_eq!(t.point([0.0; 3]).unwrap(), [1.0, 0.0, 0.0]);
        assert!(
            to_transform(&s::project_to(&s::Plane::default()), 1.0).is_err(),
            "singular"
        );
    }
}
