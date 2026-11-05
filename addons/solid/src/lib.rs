// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix-style **Solid** menu for OM9, as closed meshes:
//!
//! - primitives: Box, Sphere, Ellipsoid, Torus, Cylinder, Tube, Pyramid, Cone, Truncated
//!   Cone, Pipe, Text;
//! - booleans: Union, Difference, Intersection, Boolean Two Objects (mesh BSP booleans);
//! - from curves: Extrude Planar Curve, Extrude All (curves inside curves become holes),
//!   Boss, Rib, Slab;
//! - edits: Make Hole, Array Hole, Array Hole Polar, Move Hole, Move Face, Cap Planar Holes,
//!   Extract Surface, Shell, Fillet Edge.
//!
//! Objects from OM9's kernel (BRep) take part through their display mesh; the result is a
//! mesh object.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Replacement, Reply, Selected, SelectedMesh, Vec3, Want, add,
    dist, dot, fmt, scale, sub,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save};
use om9_addon_geom::csg::{self, Op};
use om9_addon_geom::mesh::{self, Frame, Mesh};
use om9_addon_geom::path::Target;
use om9_addon_geom::{sdf, shapes};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::f64::consts::TAU;

pub const UNION: &str = "SolidUnion";
pub const DIFFERENCE: &str = "SolidDifference";
pub const INTERSECTION: &str = "SolidIntersection";
pub const BOOLEAN_TWO: &str = "SolidBooleanTwoObjects";
pub const CAP: &str = "SolidCapPlanarHoles";
pub const EXTRACT: &str = "SolidExtractSurface";
pub const FILLET: &str = "SolidFilletEdgeFilletEdge";
pub const TEXT: &str = "SolidText";
pub const EXTRUDE: &str = "SolidExtrudePlanarCurveStraight";
pub const EXTRUDE_ALL: &str = "SolidGVDAllExtrude";
pub const PIPE: &str = "SolidPipe";
pub const BOX: &str = "SolidBoxCornertoCornerHeight";
pub const SPHERE: &str = "SolidSphereCenterRadius";
pub const ELLIPSOID: &str = "SolidEllipsoidFromCenter";
pub const TORUS: &str = "SolidTorus";
pub const CYLINDER: &str = "SolidCylinder";
pub const TUBE: &str = "SolidTube";
pub const PYRAMID: &str = "SolidPyramid";
pub const CONE: &str = "SolidCone";
pub const TRUNCATED_CONE: &str = "SolidTruncatedCone";
pub const BOSS: &str = "SolidBoss";
pub const RIB: &str = "SolidRib";
pub const SLAB: &str = "SolidSlab";
pub const MAKE_HOLE: &str = "SolidSolidEditToolsHolesMakeHole";
pub const ARRAY_HOLE: &str = "SolidSolidEditToolsHolesArrayHole";
pub const ARRAY_HOLE_POLAR: &str = "SolidSolidEditToolsHolesArrayHolePolar";
pub const MOVE_HOLE: &str = "SolidSolidEditToolsHolesMoveHole";
pub const MOVE_FACE: &str = "SolidSolidEditToolsFacesMoveFace";
pub const SHELL: &str = "SolidShell";
pub const POINTS_ON: &str = "SolidPtOn";

pub struct Solids;

impl Interactive for Solids {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        UNION,
        DIFFERENCE,
        INTERSECTION,
        BOOLEAN_TWO,
        CAP,
        EXTRACT,
        FILLET,
        TEXT,
        EXTRUDE,
        EXTRUDE_ALL,
        PIPE,
        BOX,
        SPHERE,
        ELLIPSOID,
        TORUS,
        CYLINDER,
        TUBE,
        PYRAMID,
        CONE,
        TRUNCATED_CONE,
        BOSS,
        RIB,
        SLAB,
        MAKE_HOLE,
        ARRAY_HOLE,
        ARRAY_HOLE_POLAR,
        MOVE_HOLE,
        MOVE_FACE,
        SHELL,
        POINTS_ON,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            UNION => union(e),
            DIFFERENCE => two_groups(e, Op::Difference),
            INTERSECTION => two_groups(e, Op::Intersection),
            BOOLEAN_TWO => boolean_two(e),
            CAP => cap(e),
            EXTRACT => extract(e),
            FILLET => fillet(e),
            TEXT => text(e),
            EXTRUDE | EXTRUDE_ALL => extrude(e),
            PIPE => pipe(e),
            BOX | SPHERE | ELLIPSOID | TORUS | CYLINDER | TUBE | PYRAMID | CONE
            | TRUNCATED_CONE => primitive(e),
            BOSS | RIB | SLAB => boss_rib_slab(e),
            MAKE_HOLE | ARRAY_HOLE | ARRAY_HOLE_POLAR | MOVE_HOLE => holes(e),
            MOVE_FACE => move_face(e),
            SHELL => shell(e),
            // The solid's points (mesh vertices, NURBS control points) in the point editor.
            POINTS_ON => om9_top11::edit_points(e, true, "Solid Points On"),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- helpers ---------------------------------------------------------------------------------

fn to_mesh(m: &SelectedMesh) -> Mesh {
    Mesh {
        vertices: m.vertices.clone(),
        triangles: m.triangles.clone(),
    }
}

fn solid_geo(m: Mesh, name: &str) -> Geo {
    m.geo(name, None, None)
}

/// The meshes of the selection (curves and points left out).
fn solids(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind != "curve" && s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty()))
        .collect()
}

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

fn ask_objects(prompt: String, state: serde_json::Value) -> Reply {
    Reply::ask(prompt, state).want(Want::Objects)
}

/// Replace `target` by `result` and delete the other inputs.
fn finish_replace(target: &str, result: Mesh, delete: Vec<String>, msg: String) -> Reply {
    let mut r = Reply::finish(Vec::new()).say(msg);
    r.replace.push(Replacement {
        id: target.to_owned(),
        geo: result.geo("Solid", None, None),
    });
    r.delete = delete;
    r
}

// ---- booleans --------------------------------------------------------------------------------

fn union(e: &Event) -> Reply {
    let objs = solids(e);
    if objs.len() < 2 || !matches!(e.input, Input::Enter) {
        let r = ask_objects(
            format!(
                "Boolean Union: {} solids selected. Select two or more, Enter to join",
                objs.len()
            ),
            json!({}),
        );
        return if matches!(e.input, Input::Enter) {
            r.refuse("Boolean Union: select at least two solids")
        } else {
            r
        };
    }
    let meshes: Vec<Mesh> = objs
        .iter()
        .map(|s| to_mesh(s.mesh.as_ref().unwrap()))
        .collect();
    let result = csg::fold(&meshes, Op::Union).unwrap();
    finish_replace(
        &objs[0].id,
        result,
        objs[1..].iter().map(|s| s.id.clone()).collect(),
        format!("Boolean Union: {} solids joined", objs.len()),
    )
}

#[derive(Default, Serialize, Deserialize)]
struct Groups {
    first: Vec<(String, Vec<Vec3>, Vec<[u32; 3]>)>,
}

fn two_groups(e: &Event, op: Op) -> Reply {
    let label = if op == Op::Difference {
        "Boolean Difference"
    } else {
        "Boolean Intersection"
    };
    let mut st: Groups = load(e);
    let objs = solids(e);
    if st.first.is_empty() {
        if !matches!(e.input, Input::Enter) {
            return ask_objects(
                format!(
                    "{label}: select the solids to {} ({} selected), Enter",
                    if op == Op::Difference {
                        "subtract from"
                    } else {
                        "intersect"
                    },
                    objs.len()
                ),
                save(&st),
            );
        }
        if objs.is_empty() {
            return ask_objects(format!("{label}: select solids first"), save(&st))
                .refuse(format!("{label}: nothing selected"));
        }
        st.first = objs
            .iter()
            .map(|s| {
                let m = s.mesh.as_ref().unwrap();
                (s.id.clone(), m.vertices.clone(), m.triangles.clone())
            })
            .collect();
        return ask_objects(
            format!(
                "{label}: now select the solids to {} with, Enter",
                if op == Op::Difference {
                    "subtract"
                } else {
                    "intersect"
                }
            ),
            save(&st),
        );
    }
    let ids: Vec<&String> = st.first.iter().map(|f| &f.0).collect();
    let second: Vec<&Selected> = objs.into_iter().filter(|s| !ids.contains(&&s.id)).collect();
    if second.is_empty() || !matches!(e.input, Input::Enter) {
        let r = ask_objects(
            format!(
                "{label}: {} solids in the second set. Enter to finish",
                second.len()
            ),
            save(&st),
        );
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select the second set"))
        } else {
            r
        };
    }
    let cutter = csg::fold(
        &second
            .iter()
            .map(|s| to_mesh(s.mesh.as_ref().unwrap()))
            .collect::<Vec<_>>(),
        Op::Union,
    )
    .unwrap();
    let mut r = Reply::finish(Vec::new()).say(format!(
        "{label}: {} solids with {} cutters",
        st.first.len(),
        second.len()
    ));
    for (id, v, t) in &st.first {
        let a = Mesh {
            vertices: v.clone(),
            triangles: t.clone(),
        };
        r.replace.push(Replacement {
            id: id.clone(),
            geo: csg::boolean(&a, &cutter, op).geo("Solid", None, None),
        });
    }
    r.delete = second.iter().map(|s| s.id.clone()).collect();
    r
}

fn boolean_two(e: &Event) -> Reply {
    let flow = Flow::new("Boolean Two Objects").choice(
        "Mode",
        &["Union", "Difference", "Intersection", "ReverseDifference"],
        "Difference",
    );
    let objs = solids(e);
    let compute = |c: &Collected| -> Result<Mesh, String> {
        if objs.len() != 2 {
            return Err(format!(
                "Boolean Two Objects: select exactly two solids ({} selected)",
                objs.len()
            ));
        }
        let a = to_mesh(objs[0].mesh.as_ref().unwrap());
        let b = to_mesh(objs[1].mesh.as_ref().unwrap());
        Ok(match c.opt(&flow, "Mode") {
            "Union" => csg::boolean(&a, &b, Op::Union),
            "Intersection" => csg::boolean(&a, &b, Op::Intersection),
            "ReverseDifference" => csg::boolean(&b, &a, Op::Difference),
            _ => csg::boolean(&a, &b, Op::Difference),
        })
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: compute(c).map(|m| m.wire()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let m = compute(c)?;
            Ok(finish_replace(
                &objs[0].id,
                m,
                vec![objs[1].id.clone()],
                format!("Boolean Two Objects: {}", c.opt(&flow, "Mode")),
            ))
        },
    )
}

// ---- mesh edits ------------------------------------------------------------------------------

fn cap(e: &Event) -> Reply {
    let objs = solids(e);
    if objs.is_empty() || !matches!(e.input, Input::Enter) {
        let r = ask_objects(
            format!(
                "Cap Planar Holes: {} objects. Enter caps their open edges",
                objs.len()
            ),
            json!({}),
        );
        return if matches!(e.input, Input::Enter) {
            r.refuse("Cap Planar Holes: select open meshes")
        } else {
            r
        };
    }
    let mut r = Reply::finish(Vec::new());
    let mut caps = 0;
    for s in objs {
        let (m, n) = shapes::cap_holes(&to_mesh(s.mesh.as_ref().unwrap()));
        if n > 0 {
            caps += n;
            r.replace.push(Replacement {
                id: s.id.clone(),
                geo: m.geo(&s.name, None, None),
            });
        }
    }
    r.say(format!("Cap Planar Holes: {caps} holes capped"))
}

/// Triangles of the smooth region around triangle `seed` (normals within `angle` of their
/// neighbour).
fn region(m: &SelectedMesh, seed: usize, angle: f64) -> Vec<usize> {
    let tri_n = |t: usize| {
        let tr = m.triangles[t];
        shapes::tri_normal(
            m.vertices[tr[0] as usize],
            m.vertices[tr[1] as usize],
            m.vertices[tr[2] as usize],
        )
    };
    let mut by_edge = std::collections::HashMap::<(u32, u32), Vec<usize>>::new();
    for (i, t) in m.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            by_edge.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let cos = angle.to_radians().cos();
    let mut seen = vec![false; m.triangles.len()];
    let mut stack = vec![seed];
    seen[seed] = true;
    let mut out = Vec::new();
    while let Some(t) = stack.pop() {
        out.push(t);
        let n = tri_n(t);
        let tr = m.triangles[t];
        for k in 0..3 {
            let (a, b) = (tr[k], tr[(k + 1) % 3]);
            for &o in &by_edge[&(a.min(b), a.max(b))] {
                if !seen[o] && dot(tri_n(o), n) >= cos {
                    seen[o] = true;
                    stack.push(o);
                }
            }
        }
    }
    out
}

/// The triangle of `m` under `p`, looking down the CPlane normal.
fn triangle_under(m: &SelectedMesh, p: Vec3, normal: Vec3) -> Option<usize> {
    let target = Target::new(m, normal)?;
    let (hit, _) = target.drop(add(p, scale(normal, 1e6)))?;
    (0..m.triangles.len()).min_by(|&a, &b| {
        let c = |t: usize| {
            let tr = m.triangles[t];
            let q = om9_addon_geom::sdf::closest_on_triangle(
                hit,
                m.vertices[tr[0] as usize],
                m.vertices[tr[1] as usize],
                m.vertices[tr[2] as usize],
            );
            dist(q, hit)
        };
        c(a).total_cmp(&c(b))
    })
}

fn extract(e: &Event) -> Reply {
    let flow = Flow::new("Extract Surface")
        .point("Click on the face to extract (seen from above the CPlane)");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let objs = solids(e);
        let s = objs.first().ok_or("Extract Surface: select the solid")?;
        let m = s.mesh.as_ref().unwrap();
        let seed = triangle_under(m, c.p(0).unwrap(), e.plane.normal())
            .ok_or("Extract Surface: the point is not over the solid")?;
        let face: std::collections::HashSet<usize> = region(m, seed, 20.0).into_iter().collect();
        let pick = |keep: bool| Mesh {
            vertices: m.vertices.clone(),
            triangles: m
                .triangles
                .iter()
                .enumerate()
                .filter(|(i, _)| face.contains(i) == keep)
                .map(|(_, t)| *t)
                .collect(),
        };
        let rest = shapes::weld(&pick(false), 1e-9);
        let extracted = shapes::weld(&pick(true), 1e-9);
        let mut r = Reply::finish(vec![extracted.geo("Extracted surface", None, None)]).say(
            format!("Extract Surface: {} triangles extracted", face.len()),
        );
        r.replace.push(Replacement {
            id: s.id.clone(),
            geo: rest.geo(&s.name, None, None),
        });
        Ok(r)
    })
}

fn grid_cell(m: &Mesh, feature: f64) -> f64 {
    let b = m.bbox();
    (feature / 4.0).max(dist(b[0], b[1]) / 160.0).max(0.01)
}

fn fillet(e: &Event) -> Reply {
    let flow = Flow::new("Fillet Edge").number("Fillet radius", 0.5);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let r = c.n(0).unwrap();
        if r <= 0.0 {
            return Err("the radius must be positive".into());
        }
        let objs = solids(e);
        if objs.is_empty() {
            return Err("Fillet Edge: select solids".into());
        }
        let mut out = Reply::finish(Vec::new())
            .say(format!("Fillet Edge: edges rounded with r = {}", fmt(r)));
        for s in objs {
            let m = to_mesh(s.mesh.as_ref().unwrap());
            let rounded = sdf::round_edges(&m, r, grid_cell(&m, r));
            out.replace.push(Replacement {
                id: s.id.clone(),
                geo: rounded.geo(&s.name, None, None),
            });
        }
        Ok(out)
    })
}

fn shell(e: &Event) -> Reply {
    let flow = Flow::new("Shell").number("Wall thickness", 0.5);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let t = c.n(0).unwrap();
        if t <= 0.0 {
            return Err("the thickness must be positive".into());
        }
        let objs = solids(e);
        if objs.is_empty() {
            return Err("Shell: select solids".into());
        }
        let mut out = Reply::finish(Vec::new()).say(format!("Shell: walls {} mm thick", fmt(t)));
        for s in objs {
            let m = to_mesh(s.mesh.as_ref().unwrap());
            let mut inner = sdf::offset(&m, -t, grid_cell(&m, t));
            if inner.triangles.is_empty() {
                return Err(format!("Shell: {} mm is thicker than {}", fmt(t), s.name));
            }
            for tri in &mut inner.triangles {
                tri.swap(1, 2);
            }
            let mut hollow = m.clone();
            hollow.append(&inner);
            out.replace.push(Replacement {
                id: s.id.clone(),
                geo: hollow.geo(&s.name, None, None),
            });
        }
        Ok(out)
    })
}

fn move_face(e: &Event) -> Reply {
    let flow = Flow::new("Move Face")
        .point("Click on the face to move (seen from above the CPlane)")
        .number("Distance along the face normal", 1.0);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let objs = solids(e);
        let s = objs.first().ok_or("Move Face: select the solid")?;
        let m = s.mesh.as_ref().unwrap();
        let seed = triangle_under(m, c.p(0).unwrap(), e.plane.normal())
            .ok_or("Move Face: the point is not over the solid")?;
        let face = region(m, seed, 1.0);
        let tr = m.triangles[seed];
        let n = shapes::tri_normal(
            m.vertices[tr[0] as usize],
            m.vertices[tr[1] as usize],
            m.vertices[tr[2] as usize],
        );
        let d = c.n(0).unwrap();
        let moving: std::collections::HashSet<u32> =
            face.iter().flat_map(|&t| m.triangles[t]).collect();
        let vertices = m
            .vertices
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                if moving.contains(&(i as u32)) {
                    add(v, scale(n, d))
                } else {
                    v
                }
            })
            .collect();
        let mut r = Reply::finish(Vec::new()).say(format!(
            "Move Face: {} triangles moved {} mm",
            face.len(),
            fmt(d)
        ));
        r.replace.push(Replacement {
            id: s.id.clone(),
            geo: Geo::Mesh {
                name: s.name.clone(),
                vertices,
                triangles: m.triangles.clone(),
                meta: None,
                layer: None,
            },
        });
        Ok(r)
    })
}

// ---- primitives ------------------------------------------------------------------------------

fn place(e: &Event, m: Mesh, origin: Vec3) -> Mesh {
    m.placed(&Frame::new(origin, e.plane.normal(), e.plane.x))
}

pub fn primitive_mesh(
    command: &str,
    e: &Event,
    c: &Collected,
    hover: Option<Vec3>,
    sides: usize,
) -> Option<(Mesh, &'static str)> {
    let p0 = c.p(0)?;
    let local = |p: Vec3| e.plane.local(p);
    let radius_to = |q: Vec3| {
        let a = local(p0);
        let b = local(q);
        ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
    };
    let n_or = |k: usize, dflt: Option<f64>| c.n(k).or(dflt);
    Some(match command {
        BOX => {
            let p1 = c.p(1).or(hover)?;
            let (a, b) = (local(p0), local(p1));
            let h = n_or(0, Some(1.0))?;
            let m = mesh::cuboid(
                [a[0].min(b[0]), a[1].min(b[1]), a[2] + h.min(0.0)],
                [a[0].max(b[0]), a[1].max(b[1]), a[2] + h.max(0.0)],
            );
            (
                m.placed(&Frame::new(e.plane.origin, e.plane.normal(), e.plane.x)),
                "Box",
            )
        }
        SPHERE => {
            let r = c.n(0).or_else(|| hover.map(|h| dist(p0, h)))?;
            (place(e, mesh::sphere(r, 48), p0), "Sphere")
        }
        ELLIPSOID => {
            let (a, b, cz) = (c.n(0)?, n_or(1, c.n(0))?, n_or(2, c.n(0))?);
            (place(e, mesh::ellipsoid(a, b, cz, 48), p0), "Ellipsoid")
        }
        TORUS => {
            let big = c.n(0).or_else(|| hover.map(radius_to))?;
            let small = n_or(1, Some(big * 0.2))?;
            (place(e, mesh::torus(big, small, 96, 32), p0), "Torus")
        }
        CYLINDER => {
            let r = c.n(0).or_else(|| hover.map(radius_to))?;
            let h = n_or(1, Some(r))?;
            (
                place(e, mesh::cylinder(r, h.min(0.0), h.max(0.0), 64), p0),
                "Cylinder",
            )
        }
        TUBE => {
            let ro = c.n(0).or_else(|| hover.map(radius_to))?;
            let ri = n_or(1, Some(ro * 0.8))?;
            let h = n_or(2, Some(ro))?;
            let outer = vec![mesh::circle(ro, h, 64), mesh::circle(ro, 0.0, 64)];
            let inner = vec![mesh::circle(ri, h, 64), mesh::circle(ri, 0.0, 64)];
            (place(e, mesh::tube(&outer, &inner), p0), "Tube")
        }
        PYRAMID => {
            let r = c.n(0).or_else(|| hover.map(radius_to))?;
            let h = n_or(1, Some(r))?;
            let n = sides.max(3);
            let base = mesh::circle(r, 0.0, n);
            (
                place(
                    e,
                    mesh::ring_stack(&[base], Some([0.0, 0.0, h]), Some([0.0, 0.0, 0.0])),
                    p0,
                ),
                "Pyramid",
            )
        }
        CONE => {
            let r = c.n(0).or_else(|| hover.map(radius_to))?;
            let h = n_or(1, Some(r))?;
            (
                place(e, mesh::revolve(&[(0.0, h), (r, 0.0), (0.0, 0.0)], 64), p0),
                "Cone",
            )
        }
        TRUNCATED_CONE => {
            let r0 = c.n(0).or_else(|| hover.map(radius_to))?;
            let r1 = n_or(1, Some(r0 * 0.5))?;
            let h = n_or(2, Some(r0))?;
            (
                place(
                    e,
                    mesh::revolve(&[(0.0, h), (r1, h), (r0, 0.0), (0.0, 0.0)], 64),
                    p0,
                ),
                "Truncated Cone",
            )
        }
        _ => return None,
    })
}

fn primitive(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let flow = match cmd {
        BOX => Flow::new("Box")
            .no_selection()
            .point("First corner of the base")
            .point("Other corner of the base")
            .number("Height", 1.0),
        SPHERE => Flow::new("Sphere")
            .no_selection()
            .point("Center of the sphere")
            .number("Radius", 1.0),
        ELLIPSOID => Flow::new("Ellipsoid")
            .no_selection()
            .point("Center")
            .number("Radius along the CPlane x", 2.0)
            .number("Radius along y", 1.0)
            .number("Radius along the normal", 1.0),
        TORUS => Flow::new("Torus")
            .no_selection()
            .point("Center of the torus")
            .number("Radius to the tube center", 8.0)
            .number("Tube radius", 1.0),
        CYLINDER => Flow::new("Cylinder")
            .no_selection()
            .point("Center of the base")
            .number("Radius", 1.0)
            .number("Height", 2.0),
        TUBE => Flow::new("Tube")
            .no_selection()
            .point("Center of the base")
            .number("Outer radius", 2.0)
            .number("Inner radius", 1.6)
            .number("Height", 2.0),
        PYRAMID => Flow::new("Pyramid")
            .no_selection()
            .choice("NumSides", &["3", "4", "5", "6", "8"], "4")
            .point("Center of the base")
            .number("Radius to a corner", 1.0)
            .number("Height", 1.0),
        CONE => Flow::new("Cone")
            .no_selection()
            .point("Center of the base")
            .number("Base radius", 1.0)
            .number("Height", 2.0),
        _ => Flow::new("Truncated Cone")
            .no_selection()
            .point("Center of the base")
            .number("Base radius", 2.0)
            .number("Top radius", 1.0)
            .number("Height", 2.0),
    };
    let sides = |c: &Collected| c.opt(&flow, "NumSides").parse::<usize>().unwrap_or(4);
    flow.step(
        e,
        &|c, h| Show {
            geo: primitive_mesh(cmd, e, c, h, sides(c))
                .map(|(m, _)| m.wire())
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let (m, name) = primitive_mesh(cmd, e, c, None, sides(c)).ok_or("missing value")?;
            if m.volume().abs() < 1e-12 {
                return Err(format!("{name}: the size must not be zero"));
            }
            let mut m = m;
            if m.volume() < 0.0 {
                for t in &mut m.triangles {
                    t.swap(1, 2);
                }
            }
            Ok(Reply::finish(vec![solid_geo(m, name)]).say(format!("{name}: created")))
        },
    )
}

// ---- from curves -----------------------------------------------------------------------------

/// Closed curves grouped into outlines with the curves nested inside them as holes.
fn outlines_with_holes<'a>(
    cs: &[&'a Selected],
    plane: &om9_addon_abi::session::Plane,
) -> Vec<(&'a Selected, Vec<&'a Selected>)> {
    let closed: Vec<&Selected> = cs
        .iter()
        .copied()
        .filter(|c| c.closed && c.points.len() >= 3)
        .collect();
    let flat = |s: &Selected| -> Vec<[f64; 2]> {
        s.points
            .iter()
            .map(|&p| {
                let l = plane.local(p);
                [l[0], l[1]]
            })
            .collect()
    };
    let inside = |p: [f64; 2], poly: &[[f64; 2]]| {
        let mut c = false;
        let n = poly.len();
        for i in 0..n {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                c = !c;
            }
        }
        c
    };
    let mut out = Vec::new();
    for &o in &closed {
        let po = flat(o);
        // Outlines: not inside any other closed curve.
        let nested = closed
            .iter()
            .any(|&q| q.id != o.id && inside(po[0], &flat(q)));
        if nested {
            continue;
        }
        let holes: Vec<&Selected> = closed
            .iter()
            .copied()
            .filter(|&q| q.id != o.id && inside(flat(q)[0], &po))
            .collect();
        out.push((o, holes));
    }
    out
}

fn extrude(e: &Event) -> Reply {
    let label = if e.command == EXTRUDE_ALL {
        "Extrude All"
    } else {
        "Extrude Planar Curve"
    };
    let flow = Flow::new(label)
        .toggle("BothSides", false)
        .number("Extrusion distance", 2.0);
    let n = e.plane.normal();
    let build = |c: &Collected, d: f64| -> Result<Vec<Geo>, String> {
        let cs = curves(e);
        if cs.is_empty() {
            return Err(format!("{label}: select curves"));
        }
        let both = c.yes(&flow, "BothSides");
        let (off, dir) = if both {
            (scale(n, -d), scale(n, 2.0 * d))
        } else {
            (scale(n, 0.0), scale(n, d))
        };
        let mut out = Vec::new();
        for (o, holes) in outlines_with_holes(&cs, &e.plane) {
            let shift =
                |s: &Selected| -> Vec<Vec3> { s.points.iter().map(|&p| add(p, off)).collect() };
            let hole_pts: Vec<Vec<Vec3>> = holes.iter().map(|h| shift(h)).collect();
            if let Some(m) = shapes::prism(&shift(o), &hole_pts, dir) {
                out.push(solid_geo(m, "Extrusion"));
            }
        }
        // Open curves become extruded surfaces (exact when the curve is exact).
        for s in cs.iter().filter(|s| !s.closed) {
            if let Some(Geo::Nurbs {
                degree,
                points,
                weights,
                knots,
            }) = &s.exact
            {
                let mut pts = Vec::new();
                for p in points {
                    pts.push(add(*p, off));
                    pts.push(add(add(*p, off), dir));
                }
                let w = if weights.is_empty() {
                    Vec::new()
                } else {
                    weights.iter().flat_map(|&w| [w, w]).collect()
                };
                out.push(Geo::Surface {
                    degree: [*degree, 1],
                    size: [points.len(), 2],
                    points: pts,
                    weights: w,
                    knots: [knots.clone(), vec![0.0, 0.0, 1.0, 1.0]],
                });
            } else {
                let p0: Vec<Vec3> = s.points.iter().map(|&p| add(p, off)).collect();
                let mut pts = Vec::new();
                for p in &p0 {
                    pts.push(*p);
                    pts.push(add(*p, dir));
                }
                out.push(Geo::Surface {
                    degree: [1, 1],
                    size: [p0.len(), 2],
                    knots: [
                        om9_addon_geom::clamped_uniform(p0.len(), 1),
                        vec![0.0, 0.0, 1.0, 1.0],
                    ],
                    points: pts,
                    weights: Vec::new(),
                });
            }
        }
        if out.is_empty() {
            return Err(format!(
                "{label}: no planar closed or open curves to extrude"
            ));
        }
        Ok(out)
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: c.n(0).and_then(|d| build(c, d).ok()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let d = c.n(0).unwrap();
            if d == 0.0 {
                return Err("the distance must not be zero".into());
            }
            let geo = build(c, d)?;
            let n = geo.len();
            Ok(Reply::finish(geo).say(format!("{label}: {n} objects, {} mm", fmt(d))))
        },
    )
}

fn pipe(e: &Event) -> Reply {
    let flow = Flow::new("Pipe").number("Pipe radius", 0.5);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let r = c.n(0).unwrap();
        if r <= 0.0 {
            return Err("the radius must be positive".into());
        }
        let geo: Vec<Geo> = curves(e)
            .iter()
            .filter_map(|s| mesh::pipe(&s.points, s.closed, r, 24))
            .map(|m| solid_geo(m, "Pipe"))
            .collect();
        if geo.is_empty() {
            return Err("Pipe: select curves".into());
        }
        let n = geo.len();
        Ok(Reply::finish(geo).say(format!("Pipe: {n} pipes, r = {}", fmt(r))))
    })
}

fn text(e: &Event) -> Reply {
    let flow = Flow::new("Text")
        .no_selection()
        .point("Start point of the text");
    let words = e.text("text_string", "OM9").to_owned();
    let height = e.number("text_height", 3.0).max(0.1);
    let depth = e.number("text_depth", 0.5);
    let stroke = e.number("text_stroke", 0.0);
    let make = |p: Vec3| -> Option<Mesh> {
        let f = Frame::new(p, e.plane.normal(), e.plane.x);
        let w = if stroke > 0.0 { stroke } else { height * 0.14 };
        let section = [
            [w / 2.0, depth],
            [-w / 2.0, depth],
            [-w / 2.0, 0.0],
            [w / 2.0, 0.0],
        ];
        let mut all = Mesh::default();
        for s in shapes::text_strokes(&words, &f, height, 0.25) {
            // Each stroke: a square bar laid on the CPlane, raised by the depth.
            let mut dense = Vec::new();
            for w2 in s.windows(2) {
                dense.push(w2[0]);
                dense.push(add(w2[0], scale(sub(w2[1], w2[0]), 0.5)));
            }
            dense.push(*s.last()?);
            if let Some(m) = mesh::sweep(&dense, false, e.plane.normal(), &section) {
                all.append(&m);
            }
            for &q in &[s[0], *s.last()?] {
                all.append(&mesh::cylinder(w / 2.0, 0.0, depth, 16).placed(&Frame::new(
                    q,
                    e.plane.normal(),
                    e.plane.x,
                )));
            }
        }
        (!all.triangles.is_empty()).then_some(all)
    };
    flow.step(
        e,
        &|_, h| Show {
            geo: h.and_then(make).map(|m| m.wire()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let m = make(c.p(0).unwrap()).ok_or("Text: no letters to make (A–Z, 0–9)")?;
            Ok(Reply::finish(vec![solid_geo(m, &format!("Text {words}"))])
                .say(format!("Text: \"{words}\" {} mm high", fmt(height))))
        },
    )
}

/// Boss (closed curve raised onto a solid), Rib (open curve as a wall), Slab (curve with a
/// thickness): the new part is joined to the selected solid when there is one.
fn boss_rib_slab(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        BOSS => "Boss",
        RIB => "Rib",
        _ => "Slab",
    };
    let mut flow = Flow::new(label).number("Height", 1.0);
    if cmd != BOSS {
        flow = flow.number("Thickness", 0.5);
    }
    let n = e.plane.normal();
    let part = |c: &Collected| -> Result<Mesh, String> {
        let h = c.n(0).ok_or("height")?;
        let cs = curves(e);
        let s = cs
            .first()
            .ok_or_else(|| format!("{label}: select a curve"))?;
        match cmd {
            BOSS => {
                if !s.closed {
                    return Err("Boss: the curve must be closed".into());
                }
                shapes::prism(&s.points, &[], scale(n, h))
                    .ok_or_else(|| "Boss: the curve is not planar".to_owned())
            }
            _ => {
                let t = c.n(1).ok_or("thickness")?;
                let section = [[t / 2.0, h], [-t / 2.0, h], [-t / 2.0, 0.0], [t / 2.0, 0.0]];
                mesh::sweep(&s.points, s.closed, n, &section)
                    .ok_or_else(|| format!("{label}: the curve is too short"))
            }
        }
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: part(c).map(|m| m.wire()).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let p = part(c)?;
            match solids(e).first() {
                Some(s) => Ok(finish_replace(
                    &s.id,
                    csg::boolean(&to_mesh(s.mesh.as_ref().unwrap()), &p, Op::Union),
                    Vec::new(),
                    format!("{label}: joined to {}", s.name),
                )),
                None => {
                    Ok(Reply::finish(vec![solid_geo(p, label)]).say(format!("{label}: created")))
                }
            }
        },
    )
}

/// A prism through the whole solid along the CPlane normal, from a closed curve.
fn drill(curve: &[Vec3], n: Vec3, depth: f64) -> Option<Mesh> {
    let shifted: Vec<Vec3> = curve.iter().map(|&p| add(p, scale(n, -depth))).collect();
    shapes::prism(&shifted, &[], scale(n, 2.0 * depth))
}

fn holes(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let n = e.plane.normal();
    let flow = match cmd {
        MAKE_HOLE => Flow::new("Make Hole"),
        ARRAY_HOLE => Flow::new("Array Hole")
            .number("Number in the CPlane x direction", 3.0)
            .number("Number in the CPlane y direction", 1.0)
            .number("Spacing in x", 2.0)
            .number("Spacing in y", 2.0),
        ARRAY_HOLE_POLAR => Flow::new("Array Hole Polar")
            .point("Center of the polar array")
            .number("Number of holes", 6.0)
            .number("Angle to fill", 360.0),
        _ => Flow::new("Move Hole")
            .point("Point to move the hole from")
            .point("Point to move the hole to"),
    };
    let compute = |c: &Collected| -> Result<(String, Mesh, String), String> {
        let objs = solids(e);
        let s = objs.first().ok_or("select the solid and the hole curve")?;
        let cs = curves(e);
        let hole = cs
            .iter()
            .find(|c| c.closed)
            .ok_or("select a closed curve for the hole")?;
        let depth = dist(s.bbox[0], s.bbox[1]) + 1.0;
        let base = to_mesh(s.mesh.as_ref().unwrap());
        let moved = |d: Vec3| -> Vec<Vec3> { hole.points.iter().map(|&p| add(p, d)).collect() };
        let rotated = |center: Vec3, a: f64| -> Vec<Vec3> {
            let m = om9_addon_abi::session::rotation(center, n, a);
            hole.points
                .iter()
                .map(|&p| om9_addon_abi::session::apply(&m, p))
                .collect()
        };
        let mut cutters = Vec::new();
        let mut result = base.clone();
        let msg;
        match cmd {
            MAKE_HOLE => {
                cutters.push(drill(&hole.points, n, depth).ok_or("the hole curve is not planar")?);
                msg = "Make Hole: 1 hole".to_owned();
            }
            ARRAY_HOLE => {
                let (nx, ny) = (
                    c.n(0).unwrap().round().max(1.0) as usize,
                    c.n(1).unwrap().round().max(1.0) as usize,
                );
                let (dx, dy) = (c.n(2).unwrap(), c.n(3).unwrap());
                for i in 0..nx {
                    for j in 0..ny {
                        let d = add(
                            scale(e.plane.x, i as f64 * dx),
                            scale(e.plane.y, j as f64 * dy),
                        );
                        cutters.push(
                            drill(&moved(d), n, depth).ok_or("the hole curve is not planar")?,
                        );
                    }
                }
                msg = format!("Array Hole: {} holes", nx * ny);
            }
            ARRAY_HOLE_POLAR => {
                let count = c.n(0).unwrap().round().max(1.0) as usize;
                let fill = c.n(1).unwrap();
                let step = if (fill - 360.0).abs() < 1e-9 {
                    TAU / count as f64
                } else if count > 1 {
                    fill.to_radians() / (count - 1) as f64
                } else {
                    0.0
                };
                for k in 0..count {
                    cutters.push(
                        drill(&rotated(c.p(0).unwrap(), k as f64 * step), n, depth)
                            .ok_or("the hole curve is not planar")?,
                    );
                }
                msg = format!("Array Hole Polar: {count} holes");
            }
            _ => {
                // Fill the hole where it was, cut it where it goes.
                let plug = drill(&hole.points, n, depth).ok_or("the hole curve is not planar")?;
                let plug = csg::boolean(&plug, &base_slab(&base), Op::Intersection);
                result = csg::boolean(&result, &plug, Op::Union);
                let d = sub(c.p(1).unwrap(), c.p(0).unwrap());
                cutters.push(drill(&moved(d), n, depth).ok_or("the hole curve is not planar")?);
                msg = "Move Hole: hole moved".to_owned();
            }
        }
        for cut in &cutters {
            result = csg::boolean(&result, cut, Op::Difference);
        }
        Ok((s.id.clone(), result, msg))
    };
    if cmd == MAKE_HOLE {
        if !matches!(e.input, Input::Enter) {
            return ask_objects(
                "Make Hole: select the solid and closed curves on it, Enter".into(),
                json!({}),
            );
        }
        return match compute(&Collected::default()) {
            Ok((id, m, msg)) => finish_replace(&id, m, Vec::new(), msg),
            Err(w) => ask_objects(
                "Make Hole: select the solid and a closed curve".into(),
                json!({}),
            )
            .refuse(w),
        };
    }
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (id, m, msg) = compute(c)?;
        Ok(finish_replace(&id, m, Vec::new(), msg))
    })
}

/// The box of a mesh (used to keep a plug inside the solid it fills).
fn base_slab(m: &Mesh) -> Mesh {
    let b = m.bbox();
    mesh::cuboid(b[0], b[1])
}

#[cfg(test)]
mod tests;
