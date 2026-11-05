// SPDX-License-Identifier: LGPL-2.1-or-later
//! The Matrix-style **SubD** (Clayoo) menu for OM9. A SubD object is a polygon cage (a mesh
//! of quads, written as triangle pairs); `Subdivision Level` smooths it by Catmull–Clark.
//! Creation commands can output the smooth result directly (`Levels` in the builder).
//!
//! Sub-object selection: OM9 selects whole objects, so the add-on keeps a face selection for
//! each SubD object (Select Loop, Ring, UV, Paint, Grow, Shrink, Invert, All, None, Naked,
//! Creased, Coplanar, Selection Sets). Edit commands work on those faces, or on a face picked
//! on the command line, or on the whole object. The selection is shown while a command runs
//! and stays valid until the object's cage changes outside these commands.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, Replacement, Reply, Selected, Vec3, Want, add, cross, dist,
    dot, fmt, norm, rotation, scale, sub, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, load, save};
use om9_addon_geom::mesh::{Frame, Mesh};
use om9_addon_geom::nurbs::Srf;
use om9_addon_geom::path::Path;
use om9_addon_geom::subd::{self, Edge, Poly, edge};
use om9_addon_geom::{fit, interpolate};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::TAU;
use std::sync::Mutex;

pub const ITEM: &str = "ClayooEditItem";
pub const DIVIDE: &str = "ClayooEditDivide";
pub const SPLIT_SIDES: &str = "ClayooEditSplitSides";
pub const INSET: &str = "ClayooEditInset";
pub const KNIFE: &str = "ClayooEditKnife";
pub const OFFSET: &str = "ClayooEditOffset";
pub const EXTRUDE: &str = "ClayooEditExtrude";
pub const BRIDGE: &str = "ClayooEditBridge";
pub const APPEND_FACE: &str = "ClayooCreationAppendFace";
pub const PIPE: &str = "ClayooCreationPipe";
pub const CREASE: &str = "ClayooEditCrease";
pub const FLIP: &str = "ClayooEditFlipNormals";
pub const PROJECT: &str = "ClayooEditProjectToPlane";
pub const COLLAPSE: &str = "ClayooEditCollapse";
pub const MATCH: &str = "ClayooEditMatch";
pub const WELD: &str = "ClayooEditWeld";
pub const SYMMETRY: &str = "ClayooEditSymmetry";
pub const BOX: &str = "ClayooPrimitivesBox";
pub const SPHERE: &str = "ClayooPrimitivesSphere";
pub const CYLINDER: &str = "ClayooPrimitivesCylinder";
pub const CONE: &str = "ClayooPrimitivesCone";
pub const TORUS: &str = "ClayooPrimitivesTorus";
pub const PLANE: &str = "ClayooPrimitivesPlane";
pub const SWEEP1: &str = "ClayooCreationSweep1Rail";
pub const SWEEP2: &str = "ClayooCreationSweep2Rails";
pub const LOFT: &str = "ClayooCreationLoft";
pub const WRAP: &str = "ClayooEditWrap";
pub const REVOLVE: &str = "ClayooCreationRevolve";
pub const FROM_CURVE: &str = "ClayooCreationFromCurve";
pub const FROM_2_CURVES: &str = "ClayooCreationFrom2Curves";
pub const FILL: &str = "ClayooEditFill";
pub const SHELL: &str = "ClayooEditShell";
pub const EXTRACT: &str = "ClayooEditExtract";
pub const FROM_SURFACE: &str = "ClayooCreationFromSurface";
pub const FROM_TSPLINES: &str = "ClayooCreationFromTSplines";
pub const FROM_MESH: &str = "ClayooCreationFromMesh";
pub const TO_NURBS: &str = "ClayooCreationToNurbs";
pub const SELECTION_SETS: &str = "ClayooSelectionSelectionSets";
pub const SELECT_RING: &str = "ClayooSelectionSelectRing";
pub const SELECT_LOOP: &str = "ClayooSelectionSelectLoop";
pub const SELECT_UV: &str = "ClayooSelectUV";
pub const SELECT_ALL: &str = "ClayooSelectionSelectAll";
pub const SELECT_NONE: &str = "ClayooSelectionSelectNone";
pub const PAINT: &str = "ClayooSelectionPaintSelection";
pub const GROW: &str = "ClayooSelectionGrowSelection";
pub const SHRINK: &str = "ClayooSelectionShrink";
pub const INVERT: &str = "ClayooSelectionInvertSelection";
pub const SELECT_CREASED: &str = "ClayooSelectionSelectCreased";
pub const SELECT_NAKED: &str = "ClayooSelectionSelectNaked";
pub const SELECT_COPLANAR: &str = "ClayooSelectionSelectCoplanar";
pub const LEVEL: &str = "ClayooEditSubdivisionLevel";
pub const LIBRARY: &str = "ClayooCreationClayooLibrary";
pub const SET_PLANE: &str = "ClayooSelectionSetPlane";
pub const RESET_PLANE: &str = "ClayooSelectionResetPlane";
pub const ISOCURVES: &str = "ClayooCreateIsocurves";

pub struct SubD;

impl Interactive for SubD {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        ITEM,
        DIVIDE,
        SPLIT_SIDES,
        INSET,
        KNIFE,
        OFFSET,
        EXTRUDE,
        BRIDGE,
        APPEND_FACE,
        PIPE,
        CREASE,
        FLIP,
        PROJECT,
        COLLAPSE,
        MATCH,
        WELD,
        SYMMETRY,
        BOX,
        SPHERE,
        CYLINDER,
        CONE,
        TORUS,
        PLANE,
        SWEEP1,
        SWEEP2,
        LOFT,
        WRAP,
        REVOLVE,
        FROM_CURVE,
        FROM_2_CURVES,
        FILL,
        SHELL,
        EXTRACT,
        FROM_SURFACE,
        FROM_TSPLINES,
        FROM_MESH,
        TO_NURBS,
        SELECTION_SETS,
        SELECT_RING,
        SELECT_LOOP,
        SELECT_UV,
        SELECT_ALL,
        SELECT_NONE,
        PAINT,
        GROW,
        SHRINK,
        INVERT,
        SELECT_CREASED,
        SELECT_NAKED,
        SELECT_COPLANAR,
        LEVEL,
        LIBRARY,
        SET_PLANE,
        RESET_PLANE,
        ISOCURVES,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            BOX | SPHERE | CYLINDER | CONE | TORUS | PLANE => primitive(e),
            PIPE => pipe(e),
            SWEEP1 | SWEEP2 | LOFT => from_surface_command(e),
            REVOLVE => revolve(e),
            FROM_CURVE => from_curve(e),
            FROM_2_CURVES => from_two_curves(e),
            FROM_SURFACE => from_nurbs(e),
            FROM_TSPLINES | FROM_MESH => from_mesh(e),
            TO_NURBS => to_nurbs(e),
            LIBRARY => library(e),
            ITEM => item(e),
            DIVIDE => divide(e),
            SPLIT_SIDES => split_sides(e),
            INSET | EXTRUDE | COLLAPSE | EXTRACT => face_edit(e),
            KNIFE => knife(e),
            OFFSET => offset(e),
            BRIDGE => bridge(e),
            APPEND_FACE => append_face(e),
            CREASE => crease(e),
            FLIP => whole(e, "Flip Normals", |p, _| Ok(p.flipped())),
            PROJECT => project(e),
            MATCH => match_curve(e),
            WELD => weld(e),
            SYMMETRY => symmetry(e),
            WRAP => wrap(e),
            FILL => whole(e, "Fill", |p, _| Ok(subd::fill_holes(p))),
            SHELL => shell(e),
            LEVEL => level(e),
            SELECTION_SETS => selection_sets(e),
            SELECT_RING | SELECT_LOOP | SELECT_UV | PAINT | SELECT_COPLANAR => pick_select(e),
            SELECT_ALL | SELECT_NONE | GROW | SHRINK | INVERT | SELECT_CREASED | SELECT_NAKED => {
                quick_select(e)
            }
            SET_PLANE => set_plane(e),
            RESET_PLANE => reset_plane(e),
            ISOCURVES => isocurves(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- the add-on's sub-object state -----------------------------------------------------------

/// Face selections, saved sets, creases (by position) and the SubD working plane.
struct State {
    faces: BTreeMap<String, (usize, usize, BTreeSet<usize>)>,
    sets: BTreeMap<String, Vec<usize>>,
    creases: BTreeMap<String, BTreeSet<[[i64; 3]; 2]>>,
    plane: Option<(Vec3, Vec3)>,
}

static STATE: Mutex<State> = Mutex::new(State {
    faces: BTreeMap::new(),
    sets: BTreeMap::new(),
    creases: BTreeMap::new(),
    plane: None,
});

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    let mut g = STATE.lock().unwrap_or_else(|p| p.into_inner());
    f(&mut g)
}

fn signature(p: &Poly) -> (usize, usize) {
    (p.verts.len(), p.faces.len())
}

/// The faces selected on this object (empty when its cage changed since).
pub fn selected_faces(id: &str, p: &Poly) -> BTreeSet<usize> {
    with_state(|s| match s.faces.get(id) {
        Some((v, f, set)) if (*v, *f) == signature(p) => {
            set.iter().copied().filter(|&i| i < p.faces.len()).collect()
        }
        _ => BTreeSet::new(),
    })
}

pub fn set_selected(id: &str, p: &Poly, faces: BTreeSet<usize>) {
    let (v, f) = signature(p);
    with_state(|s| {
        s.faces.insert(id.to_owned(), (v, f, faces));
    });
}

fn key(v: Vec3) -> [i64; 3] {
    [
        (v[0] * 1e5).round() as i64,
        (v[1] * 1e5).round() as i64,
        (v[2] * 1e5).round() as i64,
    ]
}

/// Creased edges of this object (found again by their end positions).
pub fn creases(id: &str, p: &Poly) -> BTreeSet<Edge> {
    let stored = with_state(|s| s.creases.get(id).cloned().unwrap_or_default());
    if stored.is_empty() {
        return BTreeSet::new();
    }
    p.edge_faces()
        .keys()
        .copied()
        .filter(|e| {
            let (a, b) = (key(p.verts[e.0 as usize]), key(p.verts[e.1 as usize]));
            stored.contains(&[a.min(b), a.max(b)])
        })
        .collect()
}

fn store_creases(id: &str, p: &Poly, edges: &BTreeSet<Edge>, add_them: bool) {
    with_state(|s| {
        let set = s.creases.entry(id.to_owned()).or_default();
        for e in edges {
            let (a, b) = (key(p.verts[e.0 as usize]), key(p.verts[e.1 as usize]));
            let k = [a.min(b), a.max(b)];
            if add_them {
                set.insert(k);
            } else {
                set.remove(&k);
            }
        }
    });
}

// ---- shared ----------------------------------------------------------------------------------

fn levels(e: &Event) -> usize {
    e.number("sd_levels", 0.0).round().clamp(0.0, 4.0) as usize
}

/// The cage wound consistently, outward when it is closed.
pub fn finalize(p: &Poly) -> Poly {
    let p = p.oriented();
    let m = p.to_mesh();
    if m.is_closed() && m.volume() < -1e-12 {
        p.flipped()
    } else {
        p
    }
}

/// A cage as the result: smoothed `levels` times (0: the cage itself).
fn out_geo(p: &Poly, levels: usize, name: &str, creased: &BTreeSet<Edge>) -> Geo {
    let p = finalize(p);
    let m = if levels == 0 {
        p.to_mesh()
    } else {
        finalize(&subd::subdivide(&p, levels, creased)).to_mesh()
    };
    m.geo(name, None, None)
}

fn curves(e: &Event) -> Vec<&Selected> {
    e.selection
        .iter()
        .filter(|s| s.kind == "curve" && s.points.len() >= 2)
        .collect()
}

/// The SubD object: the first selected mesh that is not a stone.
fn target(e: &Event) -> Option<(&Selected, Poly)> {
    e.selection
        .iter()
        .filter(|s| s.gem.is_none() && s.kind != "curve")
        .find_map(|s| {
            s.mesh
                .as_ref()
                .filter(|m| !m.triangles.is_empty())
                .map(|m| {
                    (
                        s,
                        Poly::from_mesh(&Mesh {
                            vertices: m.vertices.clone(),
                            triangles: m.triangles.clone(),
                        }),
                    )
                })
        })
}

/// The selected faces' outlines (shown while a command runs).
fn highlight(p: &Poly, faces: &BTreeSet<usize>) -> Vec<Geo> {
    faces
        .iter()
        .map(|&f| {
            let c = p.face_center(f);
            // Drawn slightly inside each face so neighbours stay apart.
            Geo::Polyline {
                points: p.faces[f]
                    .iter()
                    .map(|&i| add(c, scale(sub(p.verts[i as usize], c), 0.9)))
                    .collect(),
                closed: true,
            }
        })
        .collect()
}

fn wire(p: &Poly) -> Vec<Geo> {
    p.edge_faces()
        .keys()
        .map(|e| Geo::Line {
            start: p.verts[e.0 as usize],
            end: p.verts[e.1 as usize],
        })
        .collect()
}

fn ask_target(label: &str) -> Reply {
    Reply::ask(format!("{label}: select the SubD object, Enter"), json!({})).want(Want::Objects)
}

/// Replace the SubD object by `p` (and keep `faces` selected on the result).
fn commit(id: &str, p: &Poly, faces: Option<BTreeSet<usize>>, label: &str, what: String) -> Reply {
    let mut r = Reply::finish(Vec::new()).say(format!("{label}: {what}"));
    r.replace.push(Replacement {
        id: id.to_owned(),
        geo: out_geo(p, 0, "SubD", &BTreeSet::new()),
    });
    // The stored selection refers to the new cage as OM9 will give it back (faces of five or
    // more sides come back split, then the selection is dropped).
    let fin = finalize(p);
    let back = Poly::from_mesh(&fin.to_mesh());
    let keep = if back.faces.len() == fin.faces.len() {
        faces.unwrap_or_default()
    } else {
        BTreeSet::new()
    };
    set_selected(id, &back, keep);
    r
}

/// A whole-object edit (no options): Enter applies.
fn whole(
    e: &Event,
    label: &str,
    f: impl Fn(&Poly, &BTreeSet<usize>) -> Result<Poly, String>,
) -> Reply {
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let faces = selected_faces(&s.id, &p);
    match f(&p, &faces) {
        Ok(q) if matches!(e.input, Input::Enter) => {
            commit(&s.id, &q, None, label, format!("{} faces", q.faces.len()))
        }
        Ok(q) => Reply::ask(
            format!("{label}: {} faces. Enter to apply", q.faces.len()),
            json!({}),
        )
        .want(Want::Objects)
        .preview(wire(&q)),
        Err(w) => ask_target(label).refuse(format!("{label}: {w}")),
    }
}

fn plane_frame(e: &Event, at: Vec3) -> Frame {
    Frame::new(at, e.plane.normal(), e.plane.x)
}

// ---- primitives ------------------------------------------------------------------------------

fn faces_n(e: &Event) -> usize {
    e.number("sd_faces", 2.0).round().clamp(1.0, 32.0) as usize
}

/// A cylinder or cone cage: `sides` around, `rows` up, n-gon caps (a point tip when `top` ≈ 0).
pub fn cylinder_cage(f: &Frame, r: f64, top: f64, h: f64, sides: usize, rows: usize) -> Poly {
    let mut rings: Vec<Vec<Vec3>> = (0..=rows)
        .map(|k| {
            let t = k as f64 / rows as f64;
            let rr = r + (top - r) * t;
            (0..sides)
                .map(|i| {
                    let a = TAU * i as f64 / sides as f64;
                    f.point([rr * a.cos(), rr * a.sin(), h * t])
                })
                .collect()
        })
        .collect();
    let pointed = top < 1e-9;
    if pointed {
        rings.pop();
    }
    let mut p = subd::grid(&rings, false, true);
    let n = sides as u32;
    // Bottom cap (faces down), top cap or tip.
    p.faces.push((0..n).rev().collect());
    let last = (rings.len() as u32 - 1) * n;
    if pointed {
        p.verts.push(f.point([0.0, 0.0, h]));
        let tip = (p.verts.len() - 1) as u32;
        for i in 0..n {
            p.faces.push(vec![last + i, last + (i + 1) % n, tip]);
        }
    } else {
        p.faces.push((0..n).map(|i| last + i).collect());
    }
    p
}

pub fn torus_cage(f: &Frame, big: f64, small: f64, nu: usize, nv: usize) -> Poly {
    let rows: Vec<Vec<Vec3>> = (0..nu)
        .map(|i| {
            let a = TAU * i as f64 / nu as f64;
            (0..nv)
                .map(|j| {
                    let b = TAU * j as f64 / nv as f64;
                    let r = big + small * b.cos();
                    f.point([r * a.cos(), r * a.sin(), small * b.sin()])
                })
                .collect()
        })
        .collect();
    subd::grid(&rows, true, true)
}

pub fn sphere_cage(f: &Frame, r: f64, n: usize) -> Poly {
    let mut c = subd::box_cage([-1.0; 3], [1.0; 3], [n, n, n]);
    for v in &mut c.verts {
        *v = f.point(scale(unit(*v), r));
    }
    c
}

fn primitive(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let n = faces_n(e);
    let sides = e.number("sd_sides", 8.0).round().clamp(3.0, 64.0) as usize;
    let (label, flow) = match cmd {
        BOX => (
            "SubD Box",
            Flow::new("SubD Box")
                .no_selection()
                .point("First corner of the base")
                .point("Opposite corner")
                .number("Height", 5.0),
        ),
        SPHERE => (
            "SubD Sphere",
            Flow::new("SubD Sphere")
                .no_selection()
                .point("Center")
                .number("Radius", 5.0),
        ),
        CYLINDER => (
            "SubD Cylinder",
            Flow::new("SubD Cylinder")
                .no_selection()
                .point("Center of the base")
                .number("Radius", 3.0)
                .number("Height", 6.0),
        ),
        CONE => (
            "SubD Cone",
            Flow::new("SubD Cone")
                .no_selection()
                .point("Center of the base")
                .number("Radius", 3.0)
                .number("Height", 6.0),
        ),
        TORUS => (
            "SubD Torus",
            Flow::new("SubD Torus")
                .no_selection()
                .point("Center")
                .number("Radius to the tube's center", 8.0)
                .number("Tube radius", 1.5),
        ),
        _ => (
            "SubD Plane",
            Flow::new("SubD Plane")
                .no_selection()
                .point("First corner")
                .point("Opposite corner"),
        ),
    };
    let make = |c: &Collected| -> Option<Poly> {
        let p0 = c.p(0)?;
        let f = plane_frame(e, p0);
        Some(match cmd {
            BOX => {
                let (a, b) = (e.plane.local(p0), e.plane.local(c.p(1)?));
                let h = c.n(0).unwrap_or(5.0);
                let mut cg = subd::box_cage(
                    [a[0].min(b[0]), a[1].min(b[1]), 0.0],
                    [a[0].max(b[0]), a[1].max(b[1]), h.abs().max(1e-3)],
                    [n, n, n],
                );
                for v in &mut cg.verts {
                    *v = e.plane.at(v[0], v[1], a[2] + v[2]);
                }
                cg
            }
            SPHERE => sphere_cage(&f, c.n(0)?, n),
            CYLINDER => cylinder_cage(&f, c.n(0)?, c.n(0)?, c.n(1)?, sides, n),
            CONE => cylinder_cage(&f, c.n(0)?, 0.0, c.n(1)?, sides, n),
            TORUS => torus_cage(&f, c.n(0)?, c.n(1)?, sides * 2, sides),
            _ => {
                let (a, b) = (e.plane.local(p0), e.plane.local(c.p(1)?));
                let rows: Vec<Vec<Vec3>> = (0..=n)
                    .map(|i| {
                        (0..=n)
                            .map(|j| {
                                e.plane.at(
                                    a[0] + (b[0] - a[0]) * i as f64 / n as f64,
                                    a[1] + (b[1] - a[1]) * j as f64 / n as f64,
                                    a[2],
                                )
                            })
                            .collect()
                    })
                    .collect();
                subd::grid(&rows, false, false)
            }
        })
    };
    let lv = levels(e);
    flow.step(
        e,
        &|c, h| {
            let mut c2 = c.clone();
            if let Some(h) = h {
                // Preview with the cursor as the next point or radius.
                if c2.p(0).is_some() && matches!(cmd, BOX | PLANE) && c2.p(1).is_none() {
                    c2.answers.push(om9_addon_geom::cmd::Answer::Point(h));
                }
            }
            Show {
                geo: make(&c2).map(|p| wire(&p)).unwrap_or_default(),
                ..Default::default()
            }
        },
        &|c| {
            let p = make(c).ok_or("missing input")?;
            Ok(
                Reply::finish(vec![out_geo(&p, lv, label, &BTreeSet::new())]).say(format!(
                    "{label}: {} cage faces{}",
                    p.faces.len(),
                    if lv > 0 {
                        format!(", smoothed {lv}×")
                    } else {
                        String::new()
                    }
                )),
            )
        },
    )
    .with_label(label)
}

trait WithLabel {
    fn with_label(self, label: &str) -> Self;
}

impl WithLabel for Reply {
    fn with_label(self, _label: &str) -> Self {
        self
    }
}

// ---- from curves and surfaces ----------------------------------------------------------------

fn resample(points: &[Vec3], closed: bool, n: usize) -> Vec<Vec3> {
    om9_surfaces::resample(points, closed, n)
}

fn pipe(e: &Event) -> Reply {
    let sides = e.number("sd_sides", 8.0).round().clamp(3.0, 64.0) as usize;
    let stations = e.number("sd_stations", 12.0).round().clamp(2.0, 400.0) as usize;
    let lv = levels(e);
    let flow = Flow::new("SubD Pipe").number("Radius", 1.0);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let r = c.n(0).unwrap();
        let cs = curves(e);
        let cv = cs.first().ok_or("select a curve")?;
        let pts = resample(
            &cv.points,
            cv.closed,
            if cv.closed { stations } else { stations + 1 },
        );
        let path = Path::new(&pts, cv.closed).ok_or("curve")?;
        let rows: Vec<Vec<Vec3>> = pts
            .iter()
            .map(|&q| {
                let f = path.frame(path.closest(q), Some(e.plane.normal()));
                (0..sides)
                    .map(|i| {
                        let a = TAU * i as f64 / sides as f64;
                        add(q, add(scale(f.y, r * a.cos()), scale(f.z, r * a.sin())))
                    })
                    .collect()
            })
            .collect();
        let mut p = subd::grid(&rows, cv.closed, true);
        if !cv.closed {
            let n = sides as u32;
            let last = (rows.len() as u32 - 1) * n;
            p.faces.push((0..n).rev().collect());
            p.faces.push((0..n).map(|i| last + i).collect());
        }
        Ok(
            Reply::finish(vec![out_geo(&p, lv, "SubD pipe", &BTreeSet::new())])
                .say(format!("SubD Pipe: {} faces", p.faces.len())),
        )
    })
}

/// A cage on a NURBS surface: its points on a `nu` × `nv` grid.
fn surface_cage(g: &Geo, nu: usize, nv: usize) -> Option<Poly> {
    let s = Srf::of(g)?;
    let rows = fit::grid(&s, nu, nv);
    let closed_u = rows
        .iter()
        .zip(rows.last())
        .all(|_| dist(rows[0][nv / 2], rows[nu - 1][nv / 2]) < 1e-6);
    let closed_v = dist(rows[nu / 2][0], rows[nu / 2][nv - 1]) < 1e-6;
    let mut rows = rows;
    if closed_u {
        rows.pop();
    }
    if closed_v {
        for r in &mut rows {
            r.pop();
        }
    }
    Some(subd::grid(&rows, closed_u, closed_v))
}

fn from_surface_command(e: &Event) -> Reply {
    let (label, cmd) = match e.command.as_str() {
        SWEEP1 => ("SubD Sweep 1 Rail", om9_surfaces::SWEEP1),
        SWEEP2 => ("SubD Sweep 2 Rails", om9_surfaces::SWEEP2),
        _ => ("SubD Loft", om9_surfaces::LOFT),
    };
    let stations = e.number("sd_stations", 12.0).round().clamp(2.0, 200.0) as usize;
    let around = e.number("sd_points", 8.0).round().clamp(3.0, 200.0) as usize;
    let lv = levels(e);
    // The Surface menu builds the NURBS; the cage samples it.
    let ev = Event {
        command: cmd.to_owned(),
        input: Input::Enter,
        state: serde_json::Value::Null,
        plane: e.plane,
        selection: e.selection.clone(),
        pick: None,
        settings: BTreeMap::new(),
    };
    let made = <om9_surfaces::Surfaces as Interactive>::step(&ev)
        .ok()
        .filter(|r| r.done)
        .and_then(|r| r.create.into_iter().next());
    let Some(g) = made else {
        let r = Reply::ask(
            format!("{label}: select the curves (as for the Surface command), Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: those curves do not make a surface"))
        } else {
            r
        };
    };
    let Some(p) = surface_cage(&g, stations + 1, around + 1) else {
        return Reply::ask(label, json!({})).refuse("surface");
    };
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!("{label}: {} cage faces. Enter to create", p.faces.len()),
            json!({}),
        )
        .want(Want::Objects)
        .preview(wire(&p));
    }
    Reply::finish(vec![out_geo(&p, lv, label, &BTreeSet::new())])
        .say(format!("{label}: {} cage faces", p.faces.len()))
}

fn revolve(e: &Event) -> Reply {
    let sides = e.number("sd_sides", 8.0).round().clamp(3.0, 64.0) as usize;
    let points = e.number("sd_points", 8.0).round().clamp(2.0, 200.0) as usize;
    let lv = levels(e);
    let flow = Flow::new("SubD Revolve")
        .point("Start of the axis")
        .point("End of the axis");
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (o, b) = (c.p(0).unwrap(), c.p(1).unwrap());
        let ax = sub(b, o);
        if norm(ax) < 1e-9 {
            return Err("pick two points for the axis".into());
        }
        let cv = curves(e)
            .into_iter()
            .next()
            .ok_or("select the profile curve")?;
        let prof = resample(&cv.points, cv.closed, points);
        let rows: Vec<Vec<Vec3>> = (0..sides)
            .map(|i| {
                let m = rotation(o, ax, TAU * i as f64 / sides as f64);
                prof.iter()
                    .map(|&q| om9_addon_abi::session::apply(&m, q))
                    .collect()
            })
            .collect();
        let p = subd::grid(&rows, true, cv.closed);
        Ok(
            Reply::finish(vec![out_geo(&p, lv, "SubD revolve", &BTreeSet::new())])
                .say(format!("SubD Revolve: {} faces", p.faces.len())),
        )
    })
}

fn from_curve(e: &Event) -> Reply {
    let thick = e.number("sd_thickness", 1.0);
    let points = e.number("sd_points", 8.0).round().clamp(3.0, 200.0) as usize;
    let lv = levels(e);
    let flow = Flow::new("SubD from Curve");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let n = e.plane.normal();
        let mut out = Vec::new();
        for cv in curves(e) {
            let pts = resample(&cv.points, cv.closed, points);
            let p = if cv.closed {
                // A plate: the outline as one face each side, joined by the rim.
                let lo: Vec<Vec3> = pts.iter().map(|&q| sub(q, scale(n, thick / 2.0))).collect();
                let hi: Vec<Vec3> = pts.iter().map(|&q| add(q, scale(n, thick / 2.0))).collect();
                let mut g = subd::grid(&[lo, hi], false, true);
                let k = points as u32;
                g.faces.push((0..k).rev().collect());
                g.faces.push((0..k).map(|i| k + i).collect());
                if g.to_mesh().volume() < 0.0 {
                    g.flipped()
                } else {
                    g
                }
            } else {
                // A ribbon standing on the curve.
                subd::grid(
                    &[
                        pts.clone(),
                        pts.iter().map(|&q| add(q, scale(n, thick))).collect(),
                    ],
                    false,
                    false,
                )
            };
            out.push(out_geo(&p, lv, "SubD from curve", &BTreeSet::new()));
        }
        if out.is_empty() {
            return Err("select curves".into());
        }
        let k = out.len();
        Ok(Reply::finish(out).say(format!("SubD from Curve: {k} objects")))
    })
}

fn from_two_curves(e: &Event) -> Reply {
    let rows_n = e.number("sd_stations", 4.0).round().clamp(1.0, 200.0) as usize;
    let points = e.number("sd_points", 8.0).round().clamp(2.0, 200.0) as usize;
    let lv = levels(e);
    let flow = Flow::new("SubD from 2 Curves");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let cs = curves(e);
        if cs.len() != 2 {
            return Err("select two curves".into());
        }
        let a = resample(&cs[0].points, cs[0].closed, points);
        let mut b = resample(&cs[1].points, cs[1].closed, points);
        if !cs[0].closed && dist(a[0], b[0]) > dist(a[0], b[points - 1]) {
            b.reverse();
        }
        let rows: Vec<Vec<Vec3>> = (0..=rows_n)
            .map(|k| {
                let t = k as f64 / rows_n as f64;
                a.iter()
                    .zip(&b)
                    .map(|(p, q)| add(*p, scale(sub(*q, *p), t)))
                    .collect()
            })
            .collect();
        let p = subd::grid(&rows, false, cs[0].closed && cs[1].closed);
        Ok(Reply::finish(vec![out_geo(
            &p,
            lv,
            "SubD from 2 curves",
            &BTreeSet::new(),
        )])
        .say(format!("SubD from 2 Curves: {} faces", p.faces.len())))
    })
}

fn from_nurbs(e: &Event) -> Reply {
    let nu = e.number("sd_stations", 8.0).round().clamp(2.0, 200.0) as usize;
    let nv = e.number("sd_points", 8.0).round().clamp(2.0, 200.0) as usize;
    let lv = levels(e);
    let flow = Flow::new("SubD from Surface");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let out: Vec<Geo> = e
            .selection
            .iter()
            .filter_map(|s| s.exact.as_ref())
            .filter_map(|g| surface_cage(g, nu + 1, nv + 1))
            .map(|p| out_geo(&p, lv, "SubD from surface", &BTreeSet::new()))
            .collect();
        if out.is_empty() {
            return Err("select NURBS surfaces".into());
        }
        let k = out.len();
        Ok(Reply::finish(out).say(format!("SubD from Surface: {k} objects")))
    })
}

fn from_mesh(e: &Event) -> Reply {
    let lv = levels(e);
    let flow = Flow::new(if e.command == FROM_MESH {
        "SubD from Mesh"
    } else {
        "SubD from T-Splines"
    });
    flow.step(e, &|_, _| Show::default(), &|_| {
        let (s, p) = target(e).ok_or("select a mesh")?;
        let p = p.welded(1e-7);
        let mut r = Reply::finish(Vec::new()).say(format!(
            "SubD from Mesh: {} faces ({} quads)",
            p.faces.len(),
            p.faces.iter().filter(|f| f.len() == 4).count()
        ));
        r.replace.push(Replacement {
            id: s.id.clone(),
            geo: out_geo(&p, lv, "SubD", &BTreeSet::new()),
        });
        Ok(r)
    })
}

/// Rows of a cage that is one regular grid of quads (open in both directions), if it is.
pub fn grid_rows(p: &Poly) -> Option<Vec<Vec<Vec3>>> {
    if p.faces.iter().any(|f| f.len() != 4) {
        return None;
    }
    let ef = p.edge_faces();
    let mut adj: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for e in ef.keys() {
        adj.entry(e.0).or_default().insert(e.1);
        adj.entry(e.1).or_default().insert(e.0);
    }
    let naked: BTreeSet<Edge> = p.naked_edges().into_iter().collect();
    // A corner: two edges, both naked.
    let corner = *adj.iter().find(|(_, n)| n.len() == 2)?.0;
    let nb: Vec<u32> = adj[&corner].iter().copied().collect();
    let walk = |from: u32, toward: u32| -> Vec<u32> {
        let mut line = vec![from, toward];
        loop {
            let (a, b) = (line[line.len() - 2], line[line.len() - 1]);
            let next = adj[&b]
                .iter()
                .copied()
                .find(|&c| c != a && naked.contains(&edge(b, c)) && !line.contains(&c));
            match next {
                Some(c) if adj[&b].len() == 3 => line.push(c),
                _ => break,
            }
        }
        line
    };
    let first_row = walk(corner, nb[0]);
    let first_col = walk(corner, nb[1]);
    let (nu, nv) = (first_col.len(), first_row.len());
    if nu * nv != p.verts.len() {
        return None;
    }
    // Fill the grid row by row through the quads.
    let mut g: Vec<Vec<u32>> = vec![first_row];
    for i in 1..nu {
        let prev = g[i - 1].clone();
        let mut row = vec![first_col[i]];
        for j in 1..nv {
            let (a, b, c) = (prev[j - 1], prev[j], row[j - 1]);
            let f = ef
                .get(&edge(a, b))?
                .iter()
                .copied()
                .find(|&f| p.faces[f].contains(&c))?;
            let d = *p.faces[f].iter().find(|&&x| x != a && x != b && x != c)?;
            row.push(d);
        }
        g.push(row);
    }
    Some(
        g.into_iter()
            .map(|r| r.into_iter().map(|i| p.verts[i as usize]).collect())
            .collect(),
    )
}

fn to_nurbs(e: &Event) -> Reply {
    let flow = Flow::new("SubD to NURBS");
    flow.step(e, &|_, _| Show::default(), &|_| {
        let (s, p) = target(e).ok_or("select a SubD object")?;
        // A regular grid of quads is a uniform bicubic B-spline with the cage as control
        // net, which is what Catmull–Clark converges to.
        let rows = grid_rows(&p.welded(1e-7)).ok_or("only a cage that is one open grid of quads converts to one NURBS surface; use Subdivision Level for others")?;
        let g = fit::surface_on_grid(&rows, [3, 3]).ok_or("surface")?;
        let mut r = Reply::finish(vec![g]).say(format!("SubD to NURBS: {} × {} control points", rows.len(), rows[0].len()));
        if e.flag("sd_delete_input", false) {
            r.delete.push(s.id.clone());
        }
        Ok(r)
    })
}

/// Library shapes: outlines extruded into cages (smoothed into soft shapes).
pub fn library_shape(kind: &str, size: f64, thick: f64, f: &Frame) -> Poly {
    let n = 12;
    let outline: Vec<[f64; 2]> = (0..n)
        .map(|k| {
            let t = TAU * k as f64 / n as f64;
            match kind {
                "heart" => {
                    let (x, y) = (
                        16.0 * t.sin().powi(3),
                        13.0 * t.cos()
                            - 5.0 * (2.0 * t).cos()
                            - 2.0 * (3.0 * t).cos()
                            - (4.0 * t).cos(),
                    );
                    [x / 34.0 * size, y / 34.0 * size]
                }
                "teardrop" => [
                    size / 2.0 * t.cos() * (1.0 - t.sin()) * 0.7,
                    size / 2.0 * t.sin(),
                ],
                "leaf" => [
                    size / 2.0 * t.cos(),
                    size / 4.0 * t.sin() * (1.0 - 0.3 * t.cos()),
                ],
                "star" => {
                    let r = if k % 2 == 0 { size / 2.0 } else { size / 4.5 };
                    [r * t.cos(), r * t.sin()]
                }
                _ => [size / 2.0 * t.cos(), size / 2.0 * t.sin()],
            }
        })
        .collect();
    let lo: Vec<Vec3> = outline
        .iter()
        .map(|q| f.point([q[0], q[1], -thick / 2.0]))
        .collect();
    let hi: Vec<Vec3> = outline
        .iter()
        .map(|q| f.point([q[0], q[1], thick / 2.0]))
        .collect();
    let mid: Vec<Vec3> = outline
        .iter()
        .map(|q| f.point([q[0] * 1.08, q[1] * 1.08, 0.0]))
        .collect();
    let mut g = subd::grid(&[lo, mid, hi], false, true);
    let k = n as u32;
    g.faces.push((0..k).rev().collect());
    g.faces.push((0..k).map(|i| 2 * k + i).collect());
    if g.to_mesh().volume() < 0.0 {
        g.flipped()
    } else {
        g
    }
}

fn library(e: &Event) -> Reply {
    let kind = e.text("sd_library", "heart").to_owned();
    let size = e.number("sd_library_size", 10.0);
    let thick = e.number("sd_thickness", 3.0);
    let lv = levels(e).max(2);
    let flow = Flow::new("SubD Library")
        .no_selection()
        .point("Where to place it");
    flow.step(
        e,
        &|_, h| Show {
            geo: h
                .map(|p| wire(&library_shape(&kind, size, thick, &plane_frame(e, p))))
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(Reply::finish(vec![out_geo(
                &library_shape(&kind, size, thick, &plane_frame(e, c.p(0).unwrap())),
                lv,
                &format!("SubD {kind}"),
                &BTreeSet::new(),
            )])
            .say(format!("SubD Library: {kind} {} mm", fmt(size))))
        },
    )
}

// ---- edits -----------------------------------------------------------------------------------

/// The faces an edit works on: the stored selection, else the face picked at `p`.
fn work_faces(id: &str, poly: &Poly, pick: Option<Vec3>, e: &Event) -> BTreeSet<usize> {
    let sel = selected_faces(id, poly);
    if !sel.is_empty() {
        return sel;
    }
    pick.and_then(|p| poly.pick_face(p, e.plane.normal()))
        .into_iter()
        .collect()
}

/// Inset, Extrude, Collapse and Extract: on the selected faces, or on a picked face.
fn face_edit(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        INSET => "Inset",
        EXTRUDE => "Extrude",
        COLLAPSE => "Collapse",
        _ => "Extract",
    };
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let sel = selected_faces(&s.id, &p);
    let mut flow = Flow::new(label);
    if sel.is_empty() {
        flow = flow.point("Pick the face (or select faces with the Selection commands first)");
    }
    flow = match cmd {
        INSET => flow.number("Inset amount (fraction of the face)", 0.25),
        EXTRUDE => flow.number("Distance", 1.0),
        _ => flow,
    };
    let make = |c: &Collected| -> Result<(Poly, BTreeSet<usize>), String> {
        let faces = work_faces(&s.id, &p, c.p(0), e);
        if faces.is_empty() {
            return Err("pick a face".into());
        }
        Ok(match cmd {
            INSET => subd::inset_faces(&p, &faces, c.n(0).unwrap_or(0.25), true),
            EXTRUDE => subd::extrude_faces(&p, &faces, c.n(0).unwrap_or(1.0)),
            COLLAPSE => {
                let vs = p.face_verts(&faces);
                let cen = scale(
                    vs.iter()
                        .fold([0.0; 3], |a, &v| add(a, p.verts[v as usize])),
                    1.0 / vs.len() as f64,
                );
                let mut q = p.clone();
                for &v in &vs {
                    q.verts[v as usize] = cen;
                }
                (q.without_faces(&faces).welded(1e-9), BTreeSet::new())
            }
            _ => (p.without_faces(&faces), faces),
        })
    };
    let show = |c: &Collected, h: Option<Vec3>| -> Show {
        let mut geo = highlight(&p, &sel);
        if sel.is_empty()
            && let Some(f) = c.p(0).or(h).and_then(|q| p.pick_face(q, e.plane.normal()))
        {
            geo.extend(highlight(&p, &BTreeSet::from([f])));
        }
        if let Ok((q, _)) = make(c) {
            geo.extend(wire(&q));
        }
        Show {
            geo,
            ..Default::default()
        }
    };
    flow.step(e, &show, &|c| {
        let (q, faces) = make(c)?;
        if cmd == EXTRACT {
            // The faces become their own object; the rest stays.
            let part = Poly {
                verts: p.verts.clone(),
                faces: faces.iter().map(|&f| p.faces[f].clone()).collect(),
            }
            .compacted();
            let mut r = commit(
                &s.id,
                &q,
                None,
                label,
                format!("{} faces extracted", faces.len()),
            );
            r.create
                .push(out_geo(&part, 0, "SubD extracted", &BTreeSet::new()));
            return Ok(r);
        }
        let n = q.faces.len();
        Ok(commit(&s.id, &q, Some(faces), label, format!("{n} faces")))
    })
}

fn item(e: &Event) -> Reply {
    let label = "Edit Item";
    let flow = Flow::new(label)
        .choice("Mode", &["Move", "Scale", "Rotate"], "Move")
        .point("Base point")
        .point("Target point (Scale/Rotate: a point at the new size / angle)");
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let faces = selected_faces(&s.id, &p);
    let verts: BTreeSet<u32> = if faces.is_empty() {
        (0..p.verts.len() as u32).collect()
    } else {
        p.face_verts(&faces)
    };
    let center = scale(
        verts
            .iter()
            .fold([0.0; 3], |a, &v| add(a, p.verts[v as usize])),
        1.0 / verts.len().max(1) as f64,
    );
    let make = |c: &Collected, to: Option<Vec3>| -> Option<Poly> {
        let (a, b) = (c.p(0)?, c.p(1).or(to)?);
        let mut q = p.clone();
        let n = e.plane.normal();
        for &v in &verts {
            let x = p.verts[v as usize];
            q.verts[v as usize] = match c.opt(&flow, "Mode") {
                "Scale" => {
                    let k = dist(b, center) / dist(a, center).max(1e-9);
                    add(center, scale(sub(x, center), k))
                }
                "Rotate" => {
                    let (u, w) = (sub(a, center), sub(b, center));
                    let ang = dot(cross(u, w), n).atan2(dot(u, w));
                    om9_addon_abi::session::apply(&rotation(center, n, ang), x)
                }
                _ => add(x, sub(b, a)),
            };
        }
        Some(q)
    };
    flow.step(
        e,
        &|c, h| {
            let mut g = highlight(&p, &faces);
            g.extend(make(c, h).map(|q| wire(&q)).unwrap_or_default());
            Show {
                geo: g,
                ..Default::default()
            }
        },
        &|c| {
            Ok(commit(
                &s.id,
                &make(c, None).ok_or("pick two points")?,
                Some(faces.clone()),
                label,
                format!("{} vertices", verts.len()),
            ))
        },
    )
}

/// Split the faces in `sel` into quads around their centers; their edges get midpoints that
/// the neighbouring faces share.
pub fn divide_faces(p: &Poly, sel: &BTreeSet<usize>) -> (Poly, BTreeSet<usize>) {
    let mut out = p.clone();
    let mut mid: BTreeMap<Edge, u32> = BTreeMap::new();
    let split_edges: BTreeSet<Edge> = sel
        .iter()
        .flat_map(|&f| {
            let face = &p.faces[f];
            (0..face.len()).map(move |k| edge(face[k], face[(k + 1) % face.len()]))
        })
        .collect();
    for &e in &split_edges {
        out.verts.push(scale(
            add(p.verts[e.0 as usize], p.verts[e.1 as usize]),
            0.5,
        ));
        mid.insert(e, (out.verts.len() - 1) as u32);
    }
    let mut new_faces: Vec<Vec<u32>> = Vec::new();
    let mut new_sel = BTreeSet::new();
    for (i, face) in p.faces.iter().enumerate() {
        let n = face.len();
        if sel.contains(&i) {
            out.verts.push(p.face_center(i));
            let c = (out.verts.len() - 1) as u32;
            for k in 0..n {
                let (prev, cur, next) = (face[(k + n - 1) % n], face[k], face[(k + 1) % n]);
                new_sel.insert(new_faces.len());
                new_faces.push(vec![cur, mid[&edge(cur, next)], c, mid[&edge(prev, cur)]]);
            }
        } else {
            // Neighbours take the new midpoints into their outline.
            let mut g = Vec::new();
            for k in 0..n {
                let (a, b) = (face[k], face[(k + 1) % n]);
                g.push(a);
                if let Some(&m) = mid.get(&edge(a, b)) {
                    g.push(m);
                }
            }
            new_faces.push(g);
        }
    }
    out.faces = new_faces;
    (out, new_sel)
}

fn divide(e: &Event) -> Reply {
    whole(e, "Divide", |p, sel| {
        let all: BTreeSet<usize> = if sel.is_empty() {
            (0..p.faces.len()).collect()
        } else {
            sel.clone()
        };
        Ok(divide_faces(p, &all).0)
    })
}

fn split_sides(e: &Event) -> Reply {
    let label = "Split Sides";
    let flow = Flow::new(label)
        .number("Position along the edge (0–1)", 0.5)
        .point("Pick an edge of the ring to split");
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let make = |c: &Collected, h: Option<Vec3>| -> Option<Poly> {
        let ed = p.pick_edge(c.p(0).or(h)?, e.plane.normal())?;
        Some(subd::insert_loop(&p, ed, c.n(0).unwrap_or(0.5)))
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).map(|q| wire(&q)).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let q = make(c, None).ok_or("pick an edge")?;
            let added = q.faces.len() - p.faces.len();
            Ok(commit(
                &s.id,
                &q,
                None,
                label,
                format!("a new edge loop through {added} faces"),
            ))
        },
    )
}

/// Cut every face the plane crosses (through `a`, `b` and the CPlane normal) in two.
pub fn knife_cut(p: &Poly, a: Vec3, b: Vec3, up: Vec3) -> Poly {
    let n = unit(cross(sub(b, a), up));
    let side = |v: Vec3| dot(sub(v, a), n);
    let mut out = p.clone();
    let mut mid: BTreeMap<Edge, u32> = BTreeMap::new();
    let mut faces = Vec::new();
    for face in &p.faces {
        let k = face.len();
        let mut ring: Vec<(u32, f64)> = Vec::new();
        for i in 0..k {
            let (x, y) = (face[i], face[(i + 1) % k]);
            let (dx, dy) = (side(p.verts[x as usize]), side(p.verts[y as usize]));
            ring.push((x, dx));
            if (dx > 1e-12 && dy < -1e-12) || (dx < -1e-12 && dy > 1e-12) {
                let e = edge(x, y);
                let m = *mid.entry(e).or_insert_with(|| {
                    let t = dx / (dx - dy);
                    out.verts.push(add(
                        p.verts[x as usize],
                        scale(sub(p.verts[y as usize], p.verts[x as usize]), t),
                    ));
                    (out.verts.len() - 1) as u32
                });
                ring.push((m, 0.0));
            }
        }
        let cuts: Vec<usize> = ring
            .iter()
            .enumerate()
            .filter(|(_, (v, d))| d.abs() <= 1e-12 && !face.contains(v))
            .map(|(i, _)| i)
            .collect();
        if cuts.len() == 2 {
            let (i, j) = (cuts[0], cuts[1]);
            let f1: Vec<u32> = ring[i..=j].iter().map(|x| x.0).collect();
            let mut f2: Vec<u32> = ring[j..].iter().map(|x| x.0).collect();
            f2.extend(ring[..=i].iter().map(|x| x.0));
            faces.push(f1);
            faces.push(f2);
        } else {
            faces.push(ring.into_iter().map(|x| x.0).collect());
        }
    }
    out.faces = faces;
    out
}

fn knife(e: &Event) -> Reply {
    let label = "Knife";
    let flow = Flow::new(label)
        .point("Start of the cut")
        .point("End of the cut");
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: match (c.p(0), h) {
                (Some(a), Some(b)) => wire(&knife_cut(&p, a, b, e.plane.normal())),
                _ => Vec::new(),
            },
            ..Default::default()
        },
        &|c| {
            let q = knife_cut(&p, c.p(0).unwrap(), c.p(1).unwrap(), e.plane.normal());
            let added = q.faces.len() - p.faces.len();
            Ok(commit(&s.id, &q, None, label, format!("{added} faces cut")))
        },
    )
}

fn offset(e: &Event) -> Reply {
    let label = "Offset";
    let flow = Flow::new(label).number("Distance along the normals", 0.5);
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let faces = selected_faces(&s.id, &p);
    let make = |d: f64| -> Poly {
        let n = subd::vertex_normals(&p);
        let verts: BTreeSet<u32> = if faces.is_empty() {
            (0..p.verts.len() as u32).collect()
        } else {
            p.face_verts(&faces)
        };
        let mut q = p.clone();
        for &v in &verts {
            q.verts[v as usize] = add(p.verts[v as usize], scale(n[v as usize], d));
        }
        q
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: c.n(0).map(|d| wire(&make(d))).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(commit(
                &s.id,
                &make(c.n(0).unwrap()),
                Some(faces.clone()),
                label,
                format!("{} mm", fmt(c.n(0).unwrap())),
            ))
        },
    )
}

/// The open border loops of a cage (vertex lists, in the faces' direction reversed).
pub fn border_loops(p: &Poly) -> Vec<Vec<u32>> {
    let filled = subd::fill_holes(p);
    filled.faces[p.faces.len()..].to_vec()
}

fn bridge(e: &Event) -> Reply {
    let label = "Bridge";
    let flow = Flow::new(label).number("Segments", 2.0);
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object with two openings"))
        } else {
            r
        };
    };
    // Two openings: the open borders, or the selected faces' two groups removed.
    let sel = selected_faces(&s.id, &p);
    let base = if sel.is_empty() {
        p.clone()
    } else {
        p.without_faces(&sel)
    };
    let make = |segs: usize| -> Result<Poly, String> {
        let loops = border_loops(&base);
        if loops.len() < 2 {
            return Err("the object needs two openings (or two selected face groups)".into());
        }
        let (a, b) = (&loops[0], &loops[1]);
        if a.len() != b.len() {
            return Err(format!(
                "the openings have {} and {} edges; they must match",
                a.len(),
                b.len()
            ));
        }
        let n = a.len();
        // Align: start b at its point nearest a[0]; b runs the other way round.
        let pa = |i: usize| base.verts[a[i % n] as usize];
        let mut bb: Vec<u32> = b.iter().rev().copied().collect();
        let k = (0..n)
            .min_by(|&x, &y| {
                dist(base.verts[bb[x] as usize], pa(0))
                    .total_cmp(&dist(base.verts[bb[y] as usize], pa(0)))
            })
            .unwrap();
        bb.rotate_left(k);
        let mut q = base.clone();
        let segs = segs.max(1);
        let mut rings: Vec<Vec<u32>> = vec![a.clone()];
        for t in 1..segs {
            let f = t as f64 / segs as f64;
            let start = q.verts.len() as u32;
            for i in 0..n {
                q.verts
                    .push(add(pa(i), scale(sub(base.verts[bb[i] as usize], pa(i)), f)));
            }
            rings.push((0..n as u32).map(|i| start + i).collect());
        }
        rings.push(bb);
        for w in rings.windows(2) {
            for i in 0..n {
                let j = (i + 1) % n;
                q.faces.push(vec![w[0][j], w[0][i], w[1][i], w[1][j]]);
            }
        }
        Ok(q)
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: make(c.n(0).unwrap_or(2.0).round() as usize)
                .map(|q| wire(&q))
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let q = make(c.n(0).unwrap().round() as usize)?;
            Ok(commit(
                &s.id,
                &q,
                None,
                label,
                format!("{} faces", q.faces.len()),
            ))
        },
    )
}

fn append_face(e: &Event) -> Reply {
    let label = "Append Face";
    let flow = Flow::new(label)
        .point("Pick near an open edge")
        .point("Where the new face reaches");
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let make = |c: &Collected, to: Option<Vec3>| -> Option<Poly> {
        let a = c.p(0)?;
        let b = c.p(1).or(to)?;
        let naked = p.naked_edges();
        let n = e.plane.normal();
        let e0 = naked.iter().copied().min_by(|x, y| {
            let off = |ed: &Edge| {
                let m = scale(add(p.verts[ed.0 as usize], p.verts[ed.1 as usize]), 0.5);
                let c = sub(m, a);
                norm(sub(c, scale(n, dot(c, n))))
            };
            off(x).total_cmp(&off(y))
        })?;
        // The new quad: the edge (as its face runs it, reversed) and the edge moved to b.
        let ef = p.edge_faces();
        let f = ef[&e0][0];
        let face = &p.faces[f];
        let k = (0..face.len()).find(|&k| edge(face[k], face[(k + 1) % face.len()]) == e0)?;
        let (x, y) = (face[k], face[(k + 1) % face.len()]);
        let m = scale(add(p.verts[x as usize], p.verts[y as usize]), 0.5);
        let d = sub(b, m);
        let mut q = p.clone();
        q.verts.push(add(p.verts[y as usize], d));
        q.verts.push(add(p.verts[x as usize], d));
        let (ny, nx) = ((q.verts.len() - 2) as u32, (q.verts.len() - 1) as u32);
        q.faces.push(vec![y, x, nx, ny]);
        Some(q)
    };
    flow.step(
        e,
        &|c, h| Show {
            geo: make(c, h).map(|q| wire(&q)).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(commit(
                &s.id,
                &make(c, None).ok_or("pick an open edge")?,
                None,
                label,
                "one face added".into(),
            ))
        },
    )
}

fn crease(e: &Event) -> Reply {
    let label = "Crease";
    let flow = Flow::new(label)
        .choice("Action", &["Add", "Remove"], "Add")
        .choice("Pick", &["Loop", "Edge"], "Loop")
        .point("Pick an edge (or press Enter for the selected faces' border)");
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let current = creases(&s.id, &p);
    let faces = selected_faces(&s.id, &p);
    let pick = |c: &Collected, at: Option<Vec3>| -> BTreeSet<Edge> {
        if let Some(q) = at
            && let Some(ed) = p.pick_edge(q, e.plane.normal())
        {
            return if c.opt(&flow, "Pick") == "Loop" {
                p.edge_loop(ed).into_iter().collect()
            } else {
                BTreeSet::from([ed])
            };
        }
        // The selected faces' border.
        let mut count: BTreeMap<Edge, usize> = BTreeMap::new();
        for &f in &faces {
            let face = &p.faces[f];
            for k in 0..face.len() {
                *count
                    .entry(edge(face[k], face[(k + 1) % face.len()]))
                    .or_default() += 1;
            }
        }
        count
            .into_iter()
            .filter(|(_, n)| *n == 1)
            .map(|(e, _)| e)
            .collect()
    };
    let lines = |es: &BTreeSet<Edge>| -> Vec<Geo> {
        es.iter()
            .map(|ed| Geo::Line {
                start: p.verts[ed.0 as usize],
                end: p.verts[ed.1 as usize],
            })
            .collect()
    };
    if matches!(e.input, Input::Enter) && !faces.is_empty() {
        let c: Collected = load(e);
        let es = pick(&c, None);
        let add_them = c.opt(&flow, "Action") != "Remove";
        store_creases(&s.id, &p, &es, add_them);
        return Reply::finish(Vec::new()).say(format!(
            "{label}: {} edges {}",
            es.len(),
            if add_them { "creased" } else { "smooth again" }
        ));
    }
    flow.step(
        e,
        &|c, h| {
            let mut g = lines(&current);
            g.extend(lines(&pick(c, c.p(0).or(h))));
            Show {
                geo: g,
                ..Default::default()
            }
        },
        &|c| {
            let es = pick(c, c.p(0));
            let add_them = c.opt(&flow, "Action") != "Remove";
            store_creases(&s.id, &p, &es, add_them);
            Ok(Reply::finish(Vec::new()).say(format!(
                "{label}: {} edges {} (they stay sharp in Subdivision Level)",
                es.len(),
                if add_them { "creased" } else { "smooth again" }
            )))
        },
    )
}

fn project(e: &Event) -> Reply {
    whole(e, "Project to Plane", |p, sel| {
        let verts: BTreeSet<u32> = if sel.is_empty() {
            (0..p.verts.len() as u32).collect()
        } else {
            p.face_verts(sel)
        };
        let cen = scale(
            verts
                .iter()
                .fold([0.0; 3], |a, &v| add(a, p.verts[v as usize])),
            1.0 / verts.len() as f64,
        );
        let (o, n) = with_state(|s| s.plane).unwrap_or((cen, e.plane.normal()));
        let mut q = p.clone();
        for &v in &verts {
            let x = p.verts[v as usize];
            q.verts[v as usize] = sub(x, scale(n, dot(sub(x, o), n)));
        }
        Ok(q)
    })
}

fn match_curve(e: &Event) -> Reply {
    whole(e, "Match", |p, _| {
        let cv = curves(e)
            .into_iter()
            .next()
            .ok_or("select the SubD object and the curve to match")?;
        let path = Path::new(&cv.points, cv.closed).ok_or("curve")?;
        let loops = border_loops(p);
        let lp = loops
            .iter()
            .min_by(|a, b| {
                let d = |l: &Vec<u32>| {
                    l.iter()
                        .map(|&v| {
                            dist(
                                p.verts[v as usize],
                                path.at(path.closest(p.verts[v as usize])),
                            )
                        })
                        .sum::<f64>()
                        / l.len() as f64
                };
                d(a).total_cmp(&d(b))
            })
            .ok_or("the object has no open border")?;
        let mut q = p.clone();
        for &v in lp {
            let x = p.verts[v as usize];
            q.verts[v as usize] = path.at(path.closest(x));
        }
        Ok(q)
    })
}

fn weld(e: &Event) -> Reply {
    let tol = e.number("sd_tolerance", 0.01);
    let label = "Weld";
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select SubD objects"))
        } else {
            r
        };
    };
    // Other selected SubD objects join this one.
    let mut all = p.clone();
    let mut others = Vec::new();
    for o in e
        .selection
        .iter()
        .filter(|o| o.id != s.id && o.gem.is_none())
    {
        if let Some(m) = &o.mesh {
            let q = Poly::from_mesh(&Mesh {
                vertices: m.vertices.clone(),
                triangles: m.triangles.clone(),
            });
            let base = all.verts.len() as u32;
            all.verts.extend(q.verts);
            all.faces.extend(
                q.faces
                    .into_iter()
                    .map(|f| f.into_iter().map(|i| i + base).collect()),
            );
            others.push(o.id.clone());
        }
    }
    let q = all.welded(tol);
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(
            format!(
                "{label}: {} → {} vertices. Enter to weld",
                all.verts.len(),
                q.verts.len()
            ),
            json!({}),
        )
        .want(Want::Objects)
        .preview(wire(&q));
    }
    let mut r = commit(
        &s.id,
        &q,
        None,
        label,
        format!("{} vertices welded", all.verts.len() - q.verts.len()),
    );
    r.delete = others;
    r
}

fn symmetry(e: &Event) -> Reply {
    let axis = e.text("sd_symmetry", "x").to_owned();
    whole(e, "Symmetry", |p, _| {
        let (o, n) = with_state(|s| s.plane)
            .map(|(o, _)| (o, if axis == "y" { e.plane.y } else { e.plane.x }))
            .unwrap_or((
                e.plane.origin,
                if axis == "y" { e.plane.y } else { e.plane.x },
            ));
        let side = |v: Vec3| dot(sub(v, o), n);
        // Keep the positive half (faces with their center on it), snap the seam, mirror.
        let drop: BTreeSet<usize> = (0..p.faces.len())
            .filter(|&f| side(p.face_center(f)) < 0.0)
            .collect();
        let mut half = p.without_faces(&drop);
        for v in &mut half.verts {
            let d = side(*v);
            if d.abs() < 1e-6 || d < 0.0 {
                *v = sub(*v, scale(n, d));
            }
        }
        let mut m = half.flipped();
        for v in &mut m.verts {
            *v = sub(*v, scale(n, 2.0 * side(*v)));
        }
        let base = half.verts.len() as u32;
        let mut out = half.clone();
        out.verts.extend(m.verts);
        out.faces.extend(
            m.faces
                .into_iter()
                .map(|f| f.into_iter().map(|i| i + base).collect()),
        );
        Ok(out.welded(1e-6))
    })
}

fn wrap(e: &Event) -> Reply {
    whole(e, "Wrap", |p, sel| {
        let me = target(e).map(|t| t.0.id.clone()).unwrap_or_default();
        let other = e
            .selection
            .iter()
            .find(|s| s.id != me && s.mesh.is_some())
            .ok_or("select the SubD object, then the object to wrap onto")?;
        let m = other.mesh.as_ref().unwrap();
        let verts: BTreeSet<u32> = if sel.is_empty() {
            (0..p.verts.len() as u32).collect()
        } else {
            p.face_verts(sel)
        };
        let mut q = p.clone();
        for &v in &verts {
            if let Some(c) = om9_addon_geom::section::closest_on_mesh(m, p.verts[v as usize]) {
                q.verts[v as usize] = c;
            }
        }
        Ok(q)
    })
}

fn shell(e: &Event) -> Reply {
    let label = "Shell";
    let flow = Flow::new(label).number("Thickness", 0.5);
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    flow.step(
        e,
        &|c, _| Show {
            geo: c
                .n(0)
                .map(|t| wire(&subd::shell(&p, t)))
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(commit(
                &s.id,
                &subd::shell(&p, c.n(0).unwrap()),
                None,
                label,
                format!("{} mm", fmt(c.n(0).unwrap())),
            ))
        },
    )
}

fn level(e: &Event) -> Reply {
    let label = "Subdivision Level";
    let flow = Flow::new(label).number("Levels", 2.0);
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let cr = creases(&s.id, &p);
    let keep_cage = e.flag("sd_keep_cage", false);
    flow.step(
        e,
        &|c, _| Show {
            geo: c
                .n(0)
                .map(|n| {
                    wire(&subd::subdivide(
                        &p,
                        n.round().clamp(0.0, 4.0) as usize,
                        &cr,
                    ))
                })
                .unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            let n = c.n(0).unwrap().round().clamp(0.0, 4.0) as usize;
            let smooth = subd::subdivide(&p, n, &cr);
            if keep_cage {
                return Ok(Reply::finish(vec![out_geo(
                    &smooth,
                    0,
                    "SubD smooth",
                    &BTreeSet::new(),
                )])
                .say(format!(
                    "{label}: {} faces, the cage kept",
                    smooth.faces.len()
                )));
            }
            Ok(commit(
                &s.id,
                &smooth,
                None,
                label,
                format!(
                    "{} faces ({} creased edges kept sharp)",
                    smooth.faces.len(),
                    cr.len()
                ),
            ))
        },
    )
}

// ---- selection -------------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct Picks {
    faces: BTreeSet<usize>,
}

/// Loop, Ring, UV, Paint and Coplanar: pick faces / edges, they join the selection; Enter
/// keeps it.
fn pick_select(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        SELECT_RING => "Select Ring",
        SELECT_LOOP => "Select Loop",
        SELECT_UV => "Select UV",
        SELECT_COPLANAR => "Select Coplanar",
        _ => "Paint Selection",
    };
    let Some((s, p)) = target(e) else {
        return ask_target(label);
    };
    let mut st: Picks = load(e);
    if matches!(e.input, Input::Start) {
        st.faces = selected_faces(&s.id, &p);
    }
    let dir = e.plane.normal();
    let uv_v = e.text("sd_uv", "u") == "v";
    let angle = e.number("sd_coplanar_angle", 5.0).to_radians().cos();
    let picked = |q: Vec3| -> BTreeSet<usize> {
        match cmd {
            SELECT_RING => p
                .pick_edge(q, dir)
                .map(|ed| p.face_ring(ed).into_iter().collect())
                .unwrap_or_default(),
            SELECT_LOOP => p
                .pick_edge(q, dir)
                .map(|ed| p.face_loop(ed).into_iter().collect())
                .unwrap_or_default(),
            SELECT_UV => p
                .pick_face(q, dir)
                .map(|f| {
                    let face = &p.faces[f];
                    let k = if uv_v { 1 } else { 0 };
                    p.face_ring(edge(face[k % face.len()], face[(k + 1) % face.len()]))
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default(),
            SELECT_COPLANAR => p
                .pick_face(q, dir)
                .map(|f0| {
                    let n0 = p.face_normal(f0);
                    let ef = p.edge_faces();
                    let mut set = BTreeSet::from([f0]);
                    let mut stack = vec![f0];
                    while let Some(f) = stack.pop() {
                        let face = &p.faces[f];
                        for k in 0..face.len() {
                            for &g in &ef[&edge(face[k], face[(k + 1) % face.len()])] {
                                if !set.contains(&g) && dot(p.face_normal(g), n0) >= angle {
                                    set.insert(g);
                                    stack.push(g);
                                }
                            }
                        }
                    }
                    set
                })
                .unwrap_or_default(),
            _ => p.pick_face(q, dir).into_iter().collect(),
        }
    };
    match &e.input {
        Input::Point { point } => {
            let add_faces = picked(*point);
            // Paint toggles; the others add.
            for f in add_faces {
                if cmd == PAINT && st.faces.contains(&f) {
                    st.faces.remove(&f);
                } else {
                    st.faces.insert(f);
                }
            }
            set_selected(&s.id, &p, st.faces.clone());
        }
        Input::Enter => {
            set_selected(&s.id, &p, st.faces.clone());
            return Reply::finish(Vec::new())
                .say(format!("{label}: {} faces selected", st.faces.len()));
        }
        _ => {}
    }
    let mut preview = highlight(&p, &st.faces);
    if let Input::Hover { point } = &e.input {
        preview.extend(highlight(&p, &picked(*point)));
    }
    Reply::ask(
        format!(
            "{label}: click on the object ({} faces selected), Enter to keep",
            st.faces.len()
        ),
        save(&st),
    )
    .want(Want::Point)
    .preview(preview)
}

fn quick_select(e: &Event) -> Reply {
    let cmd = e.command.as_str();
    let label = match cmd {
        SELECT_ALL => "Select All",
        SELECT_NONE => "Select None",
        GROW => "Grow Selection",
        SHRINK => "Shrink Selection",
        INVERT => "Invert Selection",
        SELECT_CREASED => "Select Creased",
        _ => "Select Naked",
    };
    let Some((s, p)) = target(e) else {
        let r = ask_target(label);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{label}: select a SubD object"))
        } else {
            r
        };
    };
    let cur = selected_faces(&s.id, &p);
    let ef = p.edge_faces();
    let neighbours = |f: usize| -> Vec<usize> {
        let face = &p.faces[f];
        (0..face.len())
            .flat_map(|k| ef[&edge(face[k], face[(k + 1) % face.len()])].clone())
            .filter(|&g| g != f)
            .collect()
    };
    let all: BTreeSet<usize> = (0..p.faces.len()).collect();
    let next: BTreeSet<usize> = match cmd {
        SELECT_ALL => all,
        SELECT_NONE => BTreeSet::new(),
        GROW => cur
            .iter()
            .flat_map(|&f| neighbours(f))
            .chain(cur.iter().copied())
            .collect(),
        SHRINK => cur
            .iter()
            .copied()
            .filter(|&f| {
                neighbours(f).iter().all(|g| cur.contains(g))
                    && neighbours(f).len() >= p.faces[f].len()
            })
            .collect(),
        INVERT => all.difference(&cur).copied().collect(),
        SELECT_CREASED => {
            let cr = creases(&s.id, &p);
            cr.iter()
                .flat_map(|ed| ef.get(ed).cloned().unwrap_or_default())
                .chain(cur.iter().copied())
                .collect()
        }
        _ => ef
            .iter()
            .filter(|(_, f)| f.len() == 1)
            .flat_map(|(_, f)| f.clone())
            .chain(cur.iter().copied())
            .collect(),
    };
    if !matches!(e.input, Input::Enter) {
        return Reply::ask(format!("{label}: {} faces. Enter", next.len()), json!({}))
            .want(Want::Objects)
            .preview(highlight(&p, &next));
    }
    let n = next.len();
    set_selected(&s.id, &p, next);
    Reply::finish(Vec::new()).say(format!("{label}: {n} faces selected"))
}

fn selection_sets(e: &Event) -> Reply {
    let flow = Flow::new("Selection Sets").choice("Action", &["Save", "Load"], "Save");
    let name = e.text("sd_set_name", "1").to_owned();
    let Some((s, p)) = target(e) else {
        return ask_target("Selection Sets");
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let key = format!("{}#{name}", s.id);
        if c.opt(&flow, "Action") == "Load" {
            let set = with_state(|st| st.sets.get(&key).cloned())
                .ok_or(format!("no set {name} for this object"))?;
            let faces: BTreeSet<usize> = set.into_iter().filter(|&f| f < p.faces.len()).collect();
            let n = faces.len();
            set_selected(&s.id, &p, faces);
            Ok(Reply::finish(Vec::new())
                .say(format!("Selection Sets: set {name} loaded ({n} faces)")))
        } else {
            let faces = selected_faces(&s.id, &p);
            let n = faces.len();
            with_state(|st| {
                st.sets.insert(key, faces.into_iter().collect());
            });
            Ok(Reply::finish(Vec::new()).say(format!("Selection Sets: {n} faces saved as {name}")))
        }
    })
}

fn set_plane(e: &Event) -> Reply {
    let flow = Flow::new("Set SubD Plane").point("Pick the face the plane lies on");
    let Some((_, p)) = target(e) else {
        return ask_target("Set SubD Plane");
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let f = p
            .pick_face(c.p(0).unwrap(), e.plane.normal())
            .ok_or("pick a face")?;
        let (o, n) = (p.face_center(f), p.face_normal(f));
        with_state(|s| s.plane = Some((o, n)));
        let fr = Frame::new(o, n, sub(p.verts[p.faces[f][0] as usize], o));
        let sq: Vec<Vec3> = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .iter()
            .map(|q| fr.point([q[0] * 3.0, q[1] * 3.0, 0.0]))
            .collect();
        let mut r =
            Reply::finish(Vec::new()).say("Set SubD Plane: Project to Plane and Symmetry use it");
        r.preview = vec![Geo::Polyline {
            points: sq,
            closed: true,
        }];
        Ok(r)
    })
}

fn reset_plane(e: &Event) -> Reply {
    let flow = Flow::new("Reset SubD Plane").no_selection();
    flow.step(e, &|_, _| Show::default(), &|_| {
        with_state(|s| s.plane = None);
        Ok(Reply::finish(Vec::new()).say("Reset SubD Plane: back to the CPlane"))
    })
}

fn isocurves(e: &Event) -> Reply {
    let smooth = e.flag("sd_iso_smooth", true);
    let label = "Create Isocurves";
    let flow = Flow::new(label).point("Pick an edge of the loop");
    let Some((s, p)) = target(e) else {
        return ask_target(label);
    };
    let make = |q: Vec3| -> Option<Geo> {
        let ed = p.pick_edge(q, e.plane.normal())?;
        let lp = p.edge_loop(ed);
        // Order the loop's edges into a vertex chain.
        let segs: Vec<[Vec3; 2]> = lp
            .iter()
            .map(|ed| [p.verts[ed.0 as usize], p.verts[ed.1 as usize]])
            .collect();
        let (pts, closed) = om9_addon_geom::section::chain(&segs, 1e-9)
            .into_iter()
            .next()?;
        if smooth {
            interpolate(&pts, 3, closed)
        } else {
            Some(Geo::Polyline {
                points: pts,
                closed,
            })
        }
    };
    let _ = &s;
    flow.step(
        e,
        &|_, h| Show {
            geo: h.and_then(make).into_iter().collect(),
            ..Default::default()
        },
        &|c| {
            Ok(
                Reply::finish(vec![make(c.p(0).unwrap()).ok_or("pick an edge")?])
                    .say(format!("{label}: curve along the edge loop")),
            )
        },
    )
}

/// For tests: the cage of a reply's replaced object.
pub fn cage_of(g: &Geo) -> Poly {
    match g {
        Geo::Mesh {
            vertices,
            triangles,
            ..
        } => Poly::from_mesh(&Mesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
        }),
        _ => Poly::default(),
    }
}

#[cfg(test)]
mod tests;
