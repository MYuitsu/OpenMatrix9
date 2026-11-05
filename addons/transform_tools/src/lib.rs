// SPDX-License-Identifier: LGPL-2.1-or-later
//! The rest of the Matrix-style **Transform** menu for OM9 (Move, Rotate, Scale, Mirror and
//! the basic arrays are in `transform`):
//!
//! - matrices: Scale Non-Uniform, Scale by Plane, Orient 2 / 3 Points, Shear, Symmetry,
//!   Remap to CPlane, Move UVN, Make Down Facing (per stone);
//! - deformations (curves, surfaces and meshes are rebuilt in place): Bend, Taper, Twist,
//!   Stretch, Maelstrom, Soft Move, Set Points, Project to CPlane, Flow Along Curve / Surface,
//!   Smart Flow, Splop, Cage Edit;
//! - placements: Smart Flow Rigid, Smart Pattern, Array Along Curve / Surface / Curve on
//!   Surface, Orient on Surface, Orient Perpendicular to Curve, Create Cage.
//!
//! Every command follows the Matrix / Rhino flow (select objects, pick points, type values,
//! preview, Enter); results are one transaction.
use om9_addon_abi::session::{
    Event, Geo, Input, Interactive, ObjectMatrix, OptionToken, Reply, Selected, TransformOp, Vec3,
    Want, add, apply, compose, cross, dist, dot, fmt, mirror, norm, rotation, scale, scale_about,
    sub, translation, unit,
};
use om9_addon_geom::cmd::{Collected, Flow, Show, deform_selection, deformed};
use om9_addon_geom::gems::PlacedGem;
use om9_addon_geom::mesh::{self, Frame};
use om9_addon_geom::path::{Path, Target, frame_matrix};
use serde_json::json;
use std::f64::consts::PI;

pub const NONUNIFORM: &str = "TransformScaleNonUniformScale";
pub const SCALE_PLANE: &str = "TransformScaleScaleByPlane";
pub const ORIENT2: &str = "TransformOrient2Points";
pub const ORIENT3: &str = "TransformOrient3Points";
pub const SHEAR: &str = "TransformShear";
pub const SYMMETRY: &str = "TransformSymmetry";
pub const REMAP: &str = "TransformOrientRemaptoCPlane";
pub const MOVE_UVN: &str = "TransformMoveUVN";
pub const DOWN_FACING: &str = "BuilderMakeDownFacing";
pub const BEND: &str = "TransformBend";
pub const TAPER: &str = "TransformTaper";
pub const TWIST: &str = "TransformTwist";
pub const STRETCH: &str = "TransformStretch";
pub const MAELSTROM: &str = "TransformMaelstrom";
pub const SOFT_MOVE: &str = "TransformSoftMove";
pub const SET_POINTS: &str = "TransformSetPoints";
pub const PROJECT_CPLANE: &str = "TransformProjecttoCPlane";
pub const FLOW_CURVE: &str = "TransformFlowAlongCurve";
pub const FLOW_SURFACE: &str = "TransformFlowAlongSurface";
pub const SMART_FLOW: &str = "gvSmartFlow";
pub const SMART_FLOW_RIGID: &str = "gvSmartFlowRigid";
pub const SMART_FLOW_BREAK: &str = "gvSmartFlowBreak";
pub const SMART_FLOW_PULL: &str = "gvSmartFlowPullDownCurves";
pub const SMART_PATTERN: &str = "TransformSmartPattern";
pub const SPLOP: &str = "TransformSplop";
pub const CREATE_CAGE: &str = "TransformCageEditingCreateCage";
pub const CAGE_EDIT: &str = "TransformCageEditingCageEdit";
pub const ARRAY_CURVE: &str = "TransformArrayAlongCurve";
pub const ARRAY_SURFACE: &str = "TransformArrayAlongSurface";
pub const ARRAY_CURVE_SURFACE: &str = "TransformArrayAlongCurveonSurface";
pub const ORIENT_SURFACE: &str = "TransformOrientOnSurface";
pub const ORIENT_PERP: &str = "TransformOrientPerpendiculartoCurve";
pub const SOLID_POINTS: &str = "TransformSolidPtOn";

pub struct TransformTools;

impl Interactive for TransformTools {
    const SESSION_COMMANDS: &'static [&'static str] = &[
        NONUNIFORM,
        SCALE_PLANE,
        ORIENT2,
        ORIENT3,
        SHEAR,
        SYMMETRY,
        REMAP,
        MOVE_UVN,
        DOWN_FACING,
        BEND,
        TAPER,
        TWIST,
        STRETCH,
        MAELSTROM,
        SOFT_MOVE,
        SET_POINTS,
        PROJECT_CPLANE,
        FLOW_CURVE,
        FLOW_SURFACE,
        SMART_FLOW,
        SMART_FLOW_RIGID,
        SMART_FLOW_BREAK,
        SMART_FLOW_PULL,
        SMART_PATTERN,
        SPLOP,
        CREATE_CAGE,
        CAGE_EDIT,
        ARRAY_CURVE,
        ARRAY_SURFACE,
        ARRAY_CURVE_SURFACE,
        ORIENT_SURFACE,
        ORIENT_PERP,
        SOLID_POINTS,
    ];
    fn step(e: &Event) -> Result<Reply, String> {
        Ok(match e.command.as_str() {
            NONUNIFORM => nonuniform(e),
            SCALE_PLANE => scale_by_plane(e),
            ORIENT2 => orient2(e),
            ORIENT3 => orient3(e),
            SHEAR => shear(e),
            SYMMETRY => symmetry(e),
            REMAP => remap(e),
            MOVE_UVN => move_uvn(e),
            DOWN_FACING => down_facing(e),
            BEND => bend(e),
            TAPER => taper(e),
            TWIST => twist(e),
            STRETCH => stretch(e),
            MAELSTROM => maelstrom(e),
            SOFT_MOVE => soft_move(e),
            SET_POINTS => set_points(e),
            PROJECT_CPLANE => project_cplane(e),
            FLOW_CURVE => flow_along_curve(e),
            FLOW_SURFACE => flow_along_surface(e, false),
            SPLOP => flow_along_surface(e, true),
            SMART_FLOW => smart_flow(e, false),
            SMART_FLOW_RIGID => smart_flow(e, true),
            SMART_FLOW_BREAK => smart_flow_break(e),
            SMART_FLOW_PULL => pull_down(e),
            SMART_PATTERN => array_along_curve(e, "Smart Pattern"),
            ARRAY_CURVE => array_along_curve(e, "Array Along Curve"),
            ARRAY_CURVE_SURFACE => array_curve_on_surface(e),
            ARRAY_SURFACE => array_along_surface(e),
            CREATE_CAGE => create_cage(e),
            CAGE_EDIT => cage_edit(e),
            ORIENT_SURFACE => orient_on_surface(e),
            ORIENT_PERP => orient_perpendicular(e),
            SOLID_POINTS => solid_points(e),
            other => return Err(format!("unknown command {other}")),
        })
    }
}

// ---- shared helpers --------------------------------------------------------------------------

fn bbox(sel: &[&Selected]) -> Option<[Vec3; 2]> {
    let mut b = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
    for s in sel {
        for i in 0..3 {
            b[0][i] = b[0][i].min(s.bbox[0][i]);
            b[1][i] = b[1][i].max(s.bbox[1][i]);
        }
    }
    b[0][0].is_finite().then_some(b)
}

fn center(b: [Vec3; 2]) -> Vec3 {
    scale(add(b[0], b[1]), 0.5)
}

/// One matrix applied to the selection (or to a copy of it).
fn finish_matrix(m: [f64; 16], copy: bool, msg: String) -> Result<Reply, String> {
    Ok(Reply::finish_transform(TransformOp {
        matrices: vec![m],
        keep_original: copy,
        per_object: Vec::new(),
    })
    .say(msg))
}

fn show_matrix(m: Option<[f64; 16]>) -> Show {
    Show {
        transforms: m.into_iter().collect(),
        ..Default::default()
    }
}

/// A deformation finished: the selected objects rebuilt.
fn finish_deform(e: &Event, label: &str, f: &dyn Fn(Vec3) -> Vec3) -> Result<Reply, String> {
    let (replace, skipped) = deform_selection(e, f);
    if replace.is_empty() {
        return Err(format!(
            "{label}: none of the selected objects can be deformed (solids from the kernel are not supported)"
        ));
    }
    let n = replace.len();
    let mut r = Reply::finish(Vec::new()).say(format!(
        "{label}: {n} objects deformed{}",
        if skipped > 0 {
            format!(" ({skipped} skipped)")
        } else {
            String::new()
        }
    ));
    r.replace = replace;
    Ok(r)
}

/// The deformed selection as preview wires (meshes and curves).
fn show_deform(e: &Event, f: &dyn Fn(Vec3) -> Vec3) -> Show {
    let mut geo = Vec::new();
    for s in e.selection.iter().take(50) {
        if let Some(g) = deformed(s, f) {
            match g {
                Geo::Mesh {
                    vertices,
                    triangles,
                    ..
                } if triangles.len() > 20_000 => {
                    // Big meshes preview as their deformed box corners only.
                    let _ = (vertices, triangles);
                }
                g => geo.push(g),
            }
        }
    }
    Show {
        geo,
        ..Default::default()
    }
}

fn copy_flow(label: &str) -> Flow {
    Flow::new(label).toggle("Copy", false)
}

/// The curves and the other objects of the selection; the target curve is the longest.
fn split_curve_target(e: &Event) -> Option<(Path, String, Vec<&Selected>)> {
    let (curves, _): (Vec<&Selected>, Vec<&Selected>) = e
        .selection
        .iter()
        .partition(|s| s.kind == "curve" && s.points.len() >= 2);
    let target = curves.iter().max_by(|a, b| {
        Path::new(&a.points, a.closed)
            .map_or(0.0, |p| p.length())
            .total_cmp(&Path::new(&b.points, b.closed).map_or(0.0, |p| p.length()))
    })?;
    let path = Path::new(&target.points, target.closed)?;
    let others: Vec<&Selected> = e.selection.iter().filter(|s| s.id != target.id).collect();
    Some((path, target.id.clone(), others))
}

/// The surface (largest mesh-like object with triangles) and the other selected objects.
fn split_surface_target(e: &Event) -> Option<(&Selected, Vec<&Selected>)> {
    let surface = e
        .selection
        .iter()
        .filter(|s| s.mesh.as_ref().is_some_and(|m| !m.triangles.is_empty()))
        .max_by(|a, b| {
            let size = |s: &Selected| dist(s.bbox[0], s.bbox[1]);
            size(a).total_cmp(&size(b))
        })?;
    let others = e.selection.iter().filter(|s| s.id != surface.id).collect();
    Some((surface, others))
}

/// An event that sees only some of the selected objects.
fn only(e: &Event, keep: &[&Selected]) -> Event {
    let mut e2 = e.clone();
    e2.selection = keep.iter().map(|s| (*s).clone()).collect();
    e2
}

// ---- matrices --------------------------------------------------------------------------------

fn nonuniform(e: &Event) -> Reply {
    let flow = copy_flow("Scale Non-Uniform")
        .point("Origin point")
        .number("Scale factor in the CPlane x direction", 1.0)
        .number("Scale factor in the CPlane y direction", 1.0)
        .number("Scale factor in the CPlane normal direction", 1.0);
    let m = |c: &Collected| {
        Some(scale_about(
            c.p(0)?,
            &e.plane,
            [c.n(0)?, c.n(1).unwrap_or(1.0), c.n(2).unwrap_or(1.0)],
        ))
    };
    flow.step(e, &|c, _| show_matrix(m(c)), &|c| {
        let (x, y, z) = (c.n(0).unwrap(), c.n(1).unwrap(), c.n(2).unwrap());
        if x * y * z == 0.0 {
            return Err("a scale factor of 0 flattens the objects; use Project to CPlane".into());
        }
        finish_matrix(
            m(c).unwrap(),
            c.yes(&flow, "Copy"),
            format!("Scale Non-Uniform: {} × {} × {}", fmt(x), fmt(y), fmt(z)),
        )
    })
}

fn scale_by_plane(e: &Event) -> Reply {
    let flow = copy_flow("Scale by Plane")
        .point("Origin point")
        .number("Scale factor in the CPlane", 1.0);
    let m = |c: &Collected| {
        let f = c.n(0)?;
        Some(scale_about(c.p(0)?, &e.plane, [f, f, 1.0]))
    };
    flow.step(e, &|c, _| show_matrix(m(c)), &|c| {
        finish_matrix(
            m(c).ok_or("scale factor")?,
            c.yes(&flow, "Copy"),
            format!("Scale by Plane: {}", fmt(c.n(0).unwrap())),
        )
    })
}

/// Rotation taking unit direction `a` to `b` (about their common perpendicular).
fn align(a: Vec3, b: Vec3) -> [[f64; 3]; 3] {
    let (a, b) = (unit(a), unit(b));
    let v = cross(a, b);
    let c = dot(a, b);
    if norm(v) < 1e-12 {
        if c > 0.0 {
            return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        }
        // Opposite: half turn about any perpendicular.
        let p = unit(if a[0].abs() < 0.9 {
            cross(a, [1.0, 0.0, 0.0])
        } else {
            cross(a, [0.0, 1.0, 0.0])
        });
        return std::array::from_fn(|i| {
            std::array::from_fn(|j| 2.0 * p[i] * p[j] - if i == j { 1.0 } else { 0.0 })
        });
    }
    let k = 1.0 / (1.0 + c);
    let vx = [[0.0, -v[2], v[1]], [v[2], 0.0, -v[0]], [-v[1], v[0], 0.0]];
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let id = if i == j { 1.0 } else { 0.0 };
            let vx2: f64 = (0..3).map(|m| vx[i][m] * vx[m][j]).sum();
            id + vx[i][j] + vx2 * k
        })
    })
}

/// The matrix moving reference points onto target points: `p0 → t0`, `p1` toward `t1`
/// (scaled to match when `scaled`).
pub fn orient2_matrix(p0: Vec3, p1: Vec3, t0: Vec3, t1: Vec3, scaled: bool) -> Option<[f64; 16]> {
    let (a, b) = (sub(p1, p0), sub(t1, t0));
    if norm(a) < 1e-12 || norm(b) < 1e-12 {
        return None;
    }
    let s = if scaled { norm(b) / norm(a) } else { 1.0 };
    let r = align(a, b);
    let mut m = om9_addon_abi::session::IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            m[i * 4 + j] = r[i][j] * s;
        }
        m[i * 4 + 3] = t0[i] - (0..3).map(|j| r[i][j] * s * p0[j]).sum::<f64>();
    }
    Some(m)
}

fn orient2(e: &Event) -> Reply {
    let flow = copy_flow("Orient 2 Points")
        .toggle("Scale", false)
        .point("First reference point")
        .point("Second reference point")
        .point("First target point")
        .point("Second target point");
    let m = |c: &Collected, hover: Option<Vec3>| {
        let t1 = c.p(3).or(hover)?;
        orient2_matrix(c.p(0)?, c.p(1)?, c.p(2)?, t1, c.yes(&flow, "Scale"))
    };
    flow.step(e, &|c, h| show_matrix(m(c, h)), &|c| {
        finish_matrix(
            m(c, None).ok_or("the two points of a pair must differ")?,
            c.yes(&flow, "Copy"),
            "Orient 2 Points: done".into(),
        )
    })
}

fn three_point_frame(a: Vec3, b: Vec3, c: Vec3) -> Option<Frame> {
    let x = sub(b, a);
    let z = cross(x, sub(c, a));
    (norm(x) > 1e-12 && norm(z) > 1e-12).then(|| Frame::new(a, z, x))
}

fn orient3(e: &Event) -> Reply {
    let flow = copy_flow("Orient 3 Points")
        .toggle("Scale", false)
        .point("First reference point")
        .point("Second reference point")
        .point("Third reference point")
        .point("First target point")
        .point("Second target point")
        .point("Third target point");
    let m = |c: &Collected, hover: Option<Vec3>| {
        let f = three_point_frame(c.p(0)?, c.p(1)?, c.p(2)?)?;
        let t = three_point_frame(c.p(3)?, c.p(4)?, c.p(5).or(hover)?)?;
        let s = if c.yes(&flow, "Scale") {
            dist(c.p(3)?, c.p(4)?) / dist(c.p(0)?, c.p(1)?)
        } else {
            1.0
        };
        Some(frame_matrix(&f, &t, s))
    };
    flow.step(e, &|c, h| show_matrix(m(c, h)), &|c| {
        finish_matrix(
            m(c, None).ok_or("the three points must not lie on one line")?,
            c.yes(&flow, "Copy"),
            "Orient 3 Points: done".into(),
        )
    })
}

/// Shear in the CPlane: points move along `u` by tan(angle) × their distance along v.
pub fn shear_matrix(o: Vec3, reference: Vec3, angle: f64, normal: Vec3) -> Option<[f64; 16]> {
    let u = unit(sub(reference, o));
    if norm(sub(reference, o)) < 1e-12 {
        return None;
    }
    let v = unit(cross(normal, u));
    let k = angle.to_radians().tan();
    let mut m = om9_addon_abi::session::IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            m[i * 4 + j] += u[i] * k * v[j];
        }
        m[i * 4 + 3] = -(0..3).map(|j| u[i] * k * v[j] * o[j]).sum::<f64>();
    }
    Some(m)
}

fn shear(e: &Event) -> Reply {
    let flow = copy_flow("Shear")
        .point("Origin point")
        .point("Reference point (shear direction)")
        .number("Shear angle", 15.0);
    let m = |c: &Collected| shear_matrix(c.p(0)?, c.p(1)?, c.n(0)?, e.plane.normal());
    flow.step(e, &|c, _| show_matrix(m(c)), &|c| {
        if c.n(0).unwrap().abs() >= 89.0 {
            return Err("the shear angle must stay below 89°".into());
        }
        finish_matrix(
            m(c).ok_or("the reference point must differ from the origin")?,
            c.yes(&flow, "Copy"),
            format!("Shear: {}°", fmt(c.n(0).unwrap())),
        )
    })
}

fn symmetry(e: &Event) -> Reply {
    let flow = Flow::new("Symmetry")
        .choice("Axis", &["X", "Y", "XY"], "X")
        .point("Symmetry origin (Enter = CPlane origin)");
    let mats = |c: &Collected, o: Vec3| -> Vec<[f64; 16]> {
        let mx = mirror(o, e.plane.x);
        let my = mirror(o, e.plane.y);
        match c.opt(&flow, "Axis") {
            "Y" => vec![my],
            "XY" => vec![mx, my, compose(&mx, &my)],
            _ => vec![mx],
        }
    };
    if matches!(e.input, Input::Enter) {
        // Enter without a point: mirror about the CPlane origin.
        let c: Collected = om9_addon_geom::cmd::load(e);
        if c.selected && c.p(0).is_none() {
            return Reply::finish_transform(TransformOp {
                matrices: mats(&c, e.plane.origin),
                keep_original: true,
                per_object: Vec::new(),
            })
            .say("Symmetry: mirrored about the CPlane origin");
        }
    }
    flow.step(
        e,
        &|c, h| Show {
            transforms: h.map(|o| mats(c, o)).unwrap_or_default(),
            ..Default::default()
        },
        &|c| {
            Ok(Reply::finish_transform(TransformOp {
                matrices: mats(c, c.p(0).unwrap()),
                keep_original: true,
                per_object: Vec::new(),
            })
            .say(format!(
                "Symmetry: mirrored across {}",
                c.opt(&flow, "Axis")
            )))
        },
    )
}

fn remap(e: &Event) -> Reply {
    let flow = copy_flow("Remap to CPlane");
    let m = frame_matrix(
        &Frame::WORLD,
        &Frame::new(e.plane.origin, e.plane.normal(), e.plane.x),
        1.0,
    );
    let mut flow = flow;
    flow.needs.clear();
    // No points: the world top plane maps onto the active CPlane; Enter applies.
    if matches!(e.input, Input::Enter) && !e.selection.is_empty() {
        let c: Collected = om9_addon_geom::cmd::load(e);
        if c.selected {
            return Reply::finish_transform(TransformOp {
                matrices: vec![m],
                keep_original: c.yes(&flow, "Copy"),
                per_object: Vec::new(),
            })
            .say("Remap to CPlane: world top plane → active CPlane");
        }
    }
    let r = flow.step(e, &|_, _| show_matrix(Some(m)), &|_| Err(String::new()));
    if r.error.as_deref() == Some("") {
        let mut r = r;
        r.error = None;
        r.prompt = "Remap to CPlane: Enter maps the world top plane onto the active CPlane".into();
        r.want = Want::Objects;
        return r;
    }
    r
}

fn move_uvn(e: &Event) -> Reply {
    let flow = copy_flow("Move UVN")
        .number("Distance along U (CPlane x)", 0.0)
        .number("Distance along V (CPlane y)", 0.0)
        .number("Distance along N (CPlane normal)", 0.0);
    let m = |c: &Collected| {
        let d = add(
            add(
                scale(e.plane.x, c.n(0)?),
                scale(e.plane.y, c.n(1).unwrap_or(0.0)),
            ),
            scale(e.plane.normal(), c.n(2).unwrap_or(0.0)),
        );
        Some(translation(d))
    };
    flow.step(e, &|c, _| show_matrix(m(c)), &|c| {
        finish_matrix(
            m(c).unwrap(),
            c.yes(&flow, "Copy"),
            format!(
                "Move UVN: {}, {}, {}",
                fmt(c.n(0).unwrap()),
                fmt(c.n(1).unwrap()),
                fmt(c.n(2).unwrap())
            ),
        )
    })
}

/// Each stone turned upside down about its own girdle (other objects about their center).
fn down_facing(e: &Event) -> Reply {
    if e.selection.is_empty() || !matches!(e.input, Input::Enter) {
        let r = Reply::ask(
            format!(
                "Make Down Facing: {} objects. Enter turns each upside down about its girdle",
                e.selection.len()
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if e.selection.is_empty() && matches!(e.input, Input::Enter) {
            r.refuse("Make Down Facing: select stones")
        } else {
            r
        };
    }
    let per_object: Vec<ObjectMatrix> = e
        .selection
        .iter()
        .map(|s| {
            let (c, x) = match PlacedGem::from_selected(s, e.plane.normal()) {
                Some(g) => (g.frame.origin, g.frame.x),
                None => (center(s.bbox), e.plane.x),
            };
            ObjectMatrix {
                id: s.id.clone(),
                matrix: rotation(c, x, PI),
            }
        })
        .collect();
    Reply::finish_transform(TransformOp {
        matrices: Vec::new(),
        keep_original: false,
        per_object,
    })
    .say(format!(
        "Make Down Facing: {} objects turned",
        e.selection.len()
    ))
}

// ---- deformations ----------------------------------------------------------------------------

/// Bend along the spine `a → b` by `angle` degrees toward `toward` (perpendicular to the
/// spine): the spine becomes an arc of the same length.
pub fn bend_point(p: Vec3, a: Vec3, b: Vec3, toward: Vec3, angle: f64) -> Vec3 {
    let l = dist(a, b);
    let theta = angle.to_radians();
    if l < 1e-12 || theta.abs() < 1e-12 {
        return p;
    }
    let t = unit(sub(b, a));
    let d = unit(sub(toward, scale(t, dot(toward, t))));
    let w = cross(t, d);
    let q = sub(p, a);
    let (s, h, k) = (dot(q, t), dot(q, d), dot(q, w));
    let r = l / theta;
    let phi = s / r;
    let along = (r - h) * phi.sin();
    let up = r - (r - h) * phi.cos();
    add(a, add(add(scale(t, along), scale(d, up)), scale(w, k)))
}

fn bend(e: &Event) -> Reply {
    let flow = Flow::new("Bend")
        .point("Start of spine")
        .point("End of spine")
        .number("Bend angle (toward the CPlane normal)", 90.0);
    let n = e.plane.normal();
    let f = |c: &Collected| {
        let (a, b, ang) = (c.p(0), c.p(1), c.n(0));
        move |p: Vec3| match (a, b, ang) {
            (Some(a), Some(b), Some(ang)) => bend_point(p, a, b, n, ang),
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, _| {
            if c.n(0).is_some() {
                show_deform(e, &f(c))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Bend", &f(c)),
    )
}

pub fn taper_point(p: Vec3, a: Vec3, b: Vec3, start: f64, end: f64) -> Vec3 {
    let l = dist(a, b);
    if l < 1e-12 {
        return p;
    }
    let t = unit(sub(b, a));
    let q = sub(p, a);
    let s = dot(q, t);
    let radial = sub(q, scale(t, s));
    let k = start + (end - start) * (s / l);
    add(a, add(scale(t, s), scale(radial, k)))
}

fn taper(e: &Event) -> Reply {
    let flow = Flow::new("Taper")
        .point("Start of taper axis")
        .point("End of taper axis")
        .number("Scale at the start", 1.0)
        .number("Scale at the end", 0.5);
    let f = |c: &Collected| {
        let v = (c.p(0), c.p(1), c.n(0), c.n(1));
        move |p: Vec3| match v {
            (Some(a), Some(b), Some(s0), Some(s1)) => taper_point(p, a, b, s0, s1),
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, _| {
            if c.n(1).is_some() {
                show_deform(e, &f(c))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Taper", &f(c)),
    )
}

pub fn twist_point(p: Vec3, a: Vec3, b: Vec3, angle: f64) -> Vec3 {
    let l = dist(a, b);
    if l < 1e-12 {
        return p;
    }
    let t = unit(sub(b, a));
    let s = dot(sub(p, a), t);
    apply(&rotation(a, t, angle.to_radians() * s / l), p)
}

fn twist(e: &Event) -> Reply {
    let flow = Flow::new("Twist")
        .point("Start of twist axis")
        .point("End of twist axis")
        .number("Twist angle at the end", 90.0);
    let f = |c: &Collected| {
        let v = (c.p(0), c.p(1), c.n(0));
        move |p: Vec3| match v {
            (Some(a), Some(b), Some(ang)) => twist_point(p, a, b, ang),
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, _| {
            if c.n(0).is_some() {
                show_deform(e, &f(c))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Twist", &f(c)),
    )
}

fn stretch(e: &Event) -> Reply {
    let flow = Flow::new("Stretch")
        .point("Start of stretch axis")
        .point("End of stretch axis")
        .point("Point to stretch the end to");
    let f = |c: &Collected, hover: Option<Vec3>| {
        let v = (c.p(0), c.p(1), c.p(2).or(hover));
        move |p: Vec3| match v {
            (Some(a), Some(b), Some(to)) => {
                let t = unit(sub(b, a));
                let l = dist(a, b);
                if l < 1e-12 {
                    return p;
                }
                let k = dot(sub(to, a), t) / l;
                let s = dot(sub(p, a), t);
                add(p, scale(t, s * (k - 1.0)))
            }
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, h| {
            if c.p(1).is_some() {
                show_deform(e, &f(c, h))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Stretch", &f(c, None)),
    )
}

pub fn maelstrom_point(p: Vec3, c: Vec3, axis: Vec3, radius: f64, angle: f64) -> Vec3 {
    let q = sub(p, c);
    let n = unit(axis);
    let r = norm(sub(q, scale(n, dot(q, n))));
    if r >= radius || radius <= 0.0 {
        return p;
    }
    let w = (1.0 - r / radius).powi(2);
    apply(&rotation(c, n, angle.to_radians() * w), p)
}

fn maelstrom(e: &Event) -> Reply {
    let flow = Flow::new("Maelstrom")
        .point("Center of the maelstrom")
        .number("Radius", 10.0)
        .number("Angle at the center", 90.0);
    let n = e.plane.normal();
    let f = |c: &Collected| {
        let v = (c.p(0), c.n(0), c.n(1));
        move |p: Vec3| match v {
            (Some(o), Some(r), Some(a)) => maelstrom_point(p, o, n, r, a),
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, _| {
            if c.n(1).is_some() {
                show_deform(e, &f(c))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Maelstrom", &f(c)),
    )
}

pub fn soft_move_point(p: Vec3, from: Vec3, to: Vec3, radius: f64) -> Vec3 {
    let d = dist(p, from);
    if d >= radius || radius <= 0.0 {
        return p;
    }
    let k = (1.0 - (d / radius).powi(2)).powi(2);
    add(p, scale(sub(to, from), k))
}

fn soft_move(e: &Event) -> Reply {
    let flow = Flow::new("Soft Move")
        .number("Falloff radius", 5.0)
        .point("Point to move from")
        .point("Point to move to");
    let f = |c: &Collected, hover: Option<Vec3>| {
        let v = (c.n(0), c.p(0), c.p(1).or(hover));
        move |p: Vec3| match v {
            (Some(r), Some(a), Some(b)) => soft_move_point(p, a, b, r),
            _ => p,
        }
    };
    flow.step(
        e,
        &|c, h| {
            if c.p(0).is_some() {
                show_deform(e, &f(c, h))
            } else {
                Show::default()
            }
        },
        &|c| finish_deform(e, "Soft Move", &f(c, None)),
    )
}

fn set_points(e: &Event) -> Reply {
    let flow = Flow::new("Set Points")
        .toggle("X", false)
        .toggle("Y", false)
        .toggle("Z", true)
        .point("Location to set the points to");
    let f = |c: &Collected, hover: Option<Vec3>| {
        let to = c.p(0).or(hover).map(|p| e.plane.local(p));
        let (x, y, z) = (c.yes(&flow, "X"), c.yes(&flow, "Y"), c.yes(&flow, "Z"));
        move |p: Vec3| match to {
            Some(t) => {
                let mut l = e.plane.local(p);
                if x {
                    l[0] = t[0];
                }
                if y {
                    l[1] = t[1];
                }
                if z {
                    l[2] = t[2];
                }
                e.plane.at(l[0], l[1], l[2])
            }
            None => p,
        }
    };
    flow.step(e, &|c, h| show_deform(e, &f(c, h)), &|c| {
        finish_deform(e, "Set Points", &f(c, None))
    })
}

fn project_cplane(e: &Event) -> Reply {
    let f = |p: Vec3| {
        let l = e.plane.local(p);
        e.plane.at(l[0], l[1], 0.0)
    };
    if e.selection.is_empty() || !matches!(e.input, Input::Enter) {
        let r = Reply::ask(
            format!(
                "Project to CPlane: {} objects. Enter flattens them onto the CPlane",
                e.selection.len()
            ),
            json!({}),
        )
        .want(Want::Objects)
        .preview(show_deform(e, &f).geo);
        return if e.selection.is_empty() && matches!(e.input, Input::Enter) {
            r.refuse("Project to CPlane: nothing is selected")
        } else {
            r
        };
    }
    finish_deform(e, "Project to CPlane", &f).unwrap_or_else(|why| {
        Reply::ask("Project to CPlane: select objects", json!({}))
            .want(Want::Objects)
            .refuse(why)
    })
}

/// Flow from a straight base (`origin` along `axis`, length `base_len`) onto a path: the
/// point's distance along the base becomes the distance along the path (stretched to fit
/// when `stretch`), its side and height offsets follow the path's frame.
pub fn flow_point(
    p: Vec3,
    base: &Frame,
    base_len: f64,
    target: &Path,
    up: Option<Vec3>,
    stretch: bool,
) -> Vec3 {
    let l = base.local(p);
    let s = if stretch && base_len > 0.0 {
        l[0] / base_len * target.length()
    } else {
        l[0]
    };
    let f = target.frame(s, up);
    f.point([0.0, l[1], l[2]])
}

fn flow_finish(
    e: &Event,
    label: &str,
    rigid: bool,
    base: Frame,
    base_len: f64,
    target: &Path,
    target_id: &str,
    others: &[&Selected],
    stretch: bool,
    up: Option<Vec3>,
) -> Result<Reply, String> {
    let objects: Vec<&Selected> = others
        .iter()
        .copied()
        .filter(|s| s.id != target_id)
        .collect();
    if objects.is_empty() {
        return Err(format!(
            "{label}: select the objects as well as the target curve"
        ));
    }
    if rigid {
        let per_object = objects
            .iter()
            .map(|s| {
                let c = center(s.bbox);
                let l = base.local(c);
                let s_along = if stretch && base_len > 0.0 {
                    l[0] / base_len * target.length()
                } else {
                    l[0]
                };
                let from = base.moved([l[0], 0.0, 0.0]);
                let to = target.frame(s_along, up);
                ObjectMatrix {
                    id: s.id.clone(),
                    matrix: frame_matrix(&from, &to, 1.0),
                }
            })
            .collect();
        return Ok(Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: false,
            per_object,
        })
        .say(format!(
            "{label}: {} objects placed along the curve",
            objects.len()
        )));
    }
    let e2 = only(e, &objects);
    finish_deform(&e2, label, &|p| {
        flow_point(p, &base, base_len, target, up, stretch)
    })
}

fn smart_flow(e: &Event, rigid: bool) -> Reply {
    let label = if rigid {
        "Smart Flow Rigid"
    } else {
        "Smart Flow"
    };
    let flow = Flow::new(label).toggle("Stretch", true);
    let mut flow = flow;
    flow.select = true;
    let c: Collected = om9_addon_geom::cmd::load(e);
    let compute = |c: &Collected| -> Result<(Frame, f64, Path, String, Vec<&Selected>), String> {
        let (path, id, others) = split_curve_target(e).ok_or_else(|| {
            format!("{label}: select the objects and the target curve (ring rail)")
        })?;
        let objs: Vec<&Selected> = others.into_iter().collect();
        let b = bbox(&objs).ok_or_else(|| format!("{label}: select the objects to flow"))?;
        // The design lies along the CPlane x axis; its length is the base.
        let lo = e.plane.local(b[0]);
        let hi = e.plane.local(b[1]);
        let (x0, x1) = (lo[0].min(hi[0]), lo[0].max(hi[0]));
        let mid = e.plane.local(center(b));
        let origin = e.plane.at(x0, mid[1], 0.0);
        let base = Frame::new(origin, e.plane.normal(), e.plane.x);
        let _ = c;
        Ok((base, x1 - x0, path, id, objs))
    };
    match &e.input {
        Input::Enter => match compute(&c) {
            Ok((base, len, path, id, objs)) => {
                let stretch = c.yes(&flow, "Stretch");
                match flow_finish(e, label, rigid, base, len, &path, &id, &objs, stretch, None) {
                    Ok(r) => r,
                    Err(why) => flow.step(e, &|_, _| Show::default(), &|_| Err(why.clone())),
                }
            }
            Err(why) => Reply::ask(
                format!("{label}: select the objects and the target curve"),
                om9_addon_geom::cmd::save(&c),
            )
            .want(Want::Objects)
            .refuse(why),
        },
        _ => {
            let mut r = flow.step(e, &|_, _| Show::default(), &|_| Err(String::new()));
            r.error = r.error.filter(|w| !w.is_empty());
            let stretch = om9_addon_geom::cmd::load::<Collected>(&Event {
                state: r.state.clone(),
                ..e.clone()
            })
            .yes(&flow, "Stretch");
            r.prompt = match compute(&c) {
                Ok((base, len, path, id, objs)) => {
                    if !rigid {
                        let e2 = only(e, &objs);
                        r.preview =
                            show_deform(&e2, &|p| flow_point(p, &base, len, &path, None, stretch))
                                .geo;
                    }
                    let _ = id;
                    format!(
                        "{label}: {} objects onto a {} mm curve. Enter to flow",
                        objs.len(),
                        fmt(path.length())
                    )
                }
                Err(_) => format!(
                    "{label}: select the objects and the target curve (ring rail), then Enter"
                ),
            };
            r.want = Want::Objects;
            r
        }
    }
}

fn smart_flow_break(e: &Event) -> Reply {
    let mut r = Reply::finish(Vec::new());
    r.messages.push(format!(
        "Smart Flow Break: {} objects. OM9 keeps no live Smart Flow history, so flowed objects are already independent; nothing to break.",
        e.selection.len()
    ));
    r
}

fn flow_along_curve(e: &Event) -> Reply {
    let flow = Flow::new("Flow Along Curve")
        .toggle("Stretch", false)
        .toggle("Rigid", false);
    let c: Collected = om9_addon_geom::cmd::load(e);
    // Base: the selected curve nearest the objects; target: the other curve.
    let compute = || -> Result<(Path, Path, String, String, Vec<&Selected>), String> {
        let curves: Vec<&Selected> = e
            .selection
            .iter()
            .filter(|s| s.kind == "curve" && s.points.len() >= 2)
            .collect();
        let objects: Vec<&Selected> = e
            .selection
            .iter()
            .filter(|s| !(s.kind == "curve" && s.points.len() >= 2))
            .collect();
        let (curves, objects) = if curves.len() > 2 {
            // Curves being flowed: the two longest are base and target.
            let mut cs = curves.clone();
            cs.sort_by(|a, b| {
                Path::new(&b.points, b.closed)
                    .map_or(0.0, |p| p.length())
                    .total_cmp(&Path::new(&a.points, a.closed).map_or(0.0, |p| p.length()))
            });
            let rest: Vec<&Selected> = cs[2..].iter().copied().chain(objects).collect();
            (cs[..2].to_vec(), rest)
        } else {
            (curves, objects)
        };
        if curves.len() != 2 || objects.is_empty() {
            return Err(
                "Flow Along Curve: select the objects, the base curve and the target curve".into(),
            );
        }
        let b = bbox(&objects).unwrap();
        let mid = center(b);
        let d = |s: &Selected| {
            Path::new(&s.points, s.closed).map_or(f64::MAX, |p| dist(p.at(p.closest(mid)), mid))
        };
        let (base, target) = if d(curves[0]) <= d(curves[1]) {
            (curves[0], curves[1])
        } else {
            (curves[1], curves[0])
        };
        Ok((
            Path::new(&base.points, base.closed).ok_or("base curve")?,
            Path::new(&target.points, target.closed).ok_or("target curve")?,
            base.id.clone(),
            target.id.clone(),
            objects,
        ))
    };
    let map = |p: Vec3, base: &Path, target: &Path, stretch: bool| -> Vec3 {
        let s = base.closest(p);
        let bf = base.frame(s, Some(e.plane.normal()));
        let l = bf.local(p);
        let st = if stretch {
            s / base.length() * target.length()
        } else {
            s
        };
        target
            .frame(st, Some(e.plane.normal()))
            .point([0.0, l[1], l[2]])
    };
    match &e.input {
        Input::Enter => match compute() {
            Ok((base, target, _, _, objects)) => {
                let stretch = c.yes(&flow, "Stretch");
                if c.yes(&flow, "Rigid") {
                    let per_object = objects
                        .iter()
                        .map(|s| {
                            let m = center(s.bbox);
                            let sb = base.closest(m);
                            let from = base.frame(sb, Some(e.plane.normal()));
                            let st = if stretch {
                                sb / base.length() * target.length()
                            } else {
                                sb
                            };
                            ObjectMatrix {
                                id: s.id.clone(),
                                matrix: frame_matrix(
                                    &from,
                                    &target.frame(st, Some(e.plane.normal())),
                                    1.0,
                                ),
                            }
                        })
                        .collect();
                    return Reply::finish_transform(TransformOp {
                        matrices: Vec::new(),
                        keep_original: false,
                        per_object,
                    })
                    .say(format!(
                        "Flow Along Curve: {} objects placed",
                        objects.len()
                    ));
                }
                let e2 = only(e, &objects);
                finish_deform(&e2, "Flow Along Curve", &|p| {
                    map(p, &base, &target, stretch)
                })
                .unwrap_or_else(|why| {
                    Reply::ask("Flow Along Curve", om9_addon_geom::cmd::save(&c))
                        .want(Want::Objects)
                        .refuse(why)
                })
            }
            Err(why) => Reply::ask(
                "Flow Along Curve: select the objects and two curves",
                om9_addon_geom::cmd::save(&c),
            )
            .want(Want::Objects)
            .refuse(why),
        },
        _ => {
            let mut r = flow.step(e, &|_, _| Show::default(), &|_| Err(String::new()));
            r.error = r.error.filter(|w| !w.is_empty());
            r.want = Want::Objects;
            r.prompt = match compute() {
                Ok((_, target, _, _, objects)) => format!(
                    "Flow Along Curve: {} objects onto a {} mm curve. Enter to flow",
                    objects.len(),
                    fmt(target.length())
                ),
                Err(why) => why,
            };
            r
        }
    }
}

/// Points dropped along the CPlane normal onto a surface, keeping their height above it.
fn flow_along_surface(e: &Event, splop: bool) -> Reply {
    let label = if splop { "Splop" } else { "Flow Along Surface" };
    let mut flow = Flow::new(label);
    if splop {
        flow = flow
            .point("Point on the surface to splop to")
            .number("Rotation", 0.0)
            .number("Scale", 1.0);
    }
    let n = e.plane.normal();
    let prepared = || -> Result<(Target<'_>, Vec<&Selected>, Vec3), String> {
        let (surface, others) = split_surface_target(e)
            .ok_or_else(|| format!("{label}: select the objects and the target surface"))?;
        if others.is_empty() {
            return Err(format!(
                "{label}: select the objects as well as the surface"
            ));
        }
        let target = Target::new(surface.mesh.as_ref().unwrap(), n).ok_or("surface")?;
        let c = center(bbox(&others).unwrap());
        Ok((target, others, c))
    };
    let map = |target: &Target, p: Vec3, from: Vec3, to: Vec3, rot: f64, s: f64| -> Vec3 {
        // CPlane coordinates relative to the design center, moved to `to`, then dropped.
        let l = e.plane.local(p);
        let c = e.plane.local(from);
        let (dx, dy) = ((l[0] - c[0]) * s, (l[1] - c[1]) * s);
        let (sn, cs) = rot.to_radians().sin_cos();
        let (rx, ry) = (dx * cs - dy * sn, dx * sn + dy * cs);
        let t = e.plane.local(to);
        let probe = e.plane.at(t[0] + rx, t[1] + ry, 1e6);
        match target.drop(probe) {
            // The height above the design's base becomes the height above the surface.
            Some((hit, normal)) => add(hit, scale(normal, (l[2] - c[2]) * s)),
            None => p,
        }
    };
    let finish = |c: &Collected| -> Result<Reply, String> {
        let (target, objs, mid) = prepared()?;
        let to = if splop { c.p(0).unwrap() } else { mid };
        let (rot, s) = if splop {
            (c.n(0).unwrap(), c.n(1).unwrap())
        } else {
            (0.0, 1.0)
        };
        // Heights are measured from the design's lowest point.
        let low = bbox(&objs).unwrap()[0];
        let base = e.plane.at(
            e.plane.local(mid)[0],
            e.plane.local(mid)[1],
            e.plane.local(low)[2],
        );
        let e2 = only(e, &objs);
        finish_deform(&e2, label, &|p| map(&target, p, base, to, rot, s))
    };
    if !splop {
        if matches!(e.input, Input::Enter) && !e.selection.is_empty() {
            return finish(&Collected::default()).unwrap_or_else(|why| {
                Reply::ask(
                    format!("{label}: select the objects and the surface"),
                    json!({}),
                )
                .want(Want::Objects)
                .refuse(why)
            });
        }
        let prompt = match prepared() {
            Ok((_, objs, _)) => format!(
                "{label}: {} objects onto the surface. Enter to flow",
                objs.len()
            ),
            Err(why) => why,
        };
        return Reply::ask(prompt, json!({})).want(Want::Objects);
    }
    flow.step(e, &|_, _| Show::default(), &finish)
}

fn pull_down(e: &Event) -> Reply {
    let n = e.plane.normal();
    let compute = || -> Result<(Target<'_>, Vec<&Selected>), String> {
        let (surface, others) = split_surface_target(e)
            .ok_or("Smart Flow Pull Down Curves: select the curves and the surface")?;
        let curves: Vec<&Selected> = others.into_iter().filter(|s| s.kind == "curve").collect();
        if curves.is_empty() {
            return Err("Smart Flow Pull Down Curves: select curves above the surface".into());
        }
        Ok((
            Target::new(surface.mesh.as_ref().unwrap(), n).ok_or("surface")?,
            curves,
        ))
    };
    if !matches!(e.input, Input::Enter) {
        let prompt = match compute() {
            Ok((_, c)) => format!(
                "Smart Flow Pull Down Curves: {} curves. Enter pulls them onto the surface",
                c.len()
            ),
            Err(w) => w,
        };
        return Reply::ask(prompt, json!({})).want(Want::Objects);
    }
    match compute() {
        Ok((target, curves)) => {
            let e2 = only(e, &curves);
            finish_deform(&e2, "Smart Flow Pull Down Curves", &|p| {
                target.drop(add(p, scale(n, 1e6))).map_or(p, |h| h.0)
            })
            .unwrap_or_else(|w| {
                Reply::ask(
                    "Smart Flow Pull Down Curves: select the curves and the target, Enter",
                    json!({}),
                )
                .refuse(w)
            })
        }
        Err(w) => Reply::ask(
            "Smart Flow Pull Down Curves: select the curves and the target, Enter",
            json!({}),
        )
        .want(Want::Objects)
        .refuse(w),
    }
}

// ---- placements ------------------------------------------------------------------------------

fn array_along_curve(e: &Event, label: &str) -> Reply {
    let flow = Flow::new(label)
        .toggle("Orient", true)
        .number("Number of items", 8.0);
    let plan = |count: usize, orient: bool| -> Result<(Vec<ObjectMatrix>, usize), String> {
        let (path, id, objects) = split_curve_target(e)
            .ok_or_else(|| format!("{label}: select the objects and the curve"))?;
        if objects.is_empty() {
            return Err(format!("{label}: select the objects as well as the curve"));
        }
        let b = bbox(&objects).unwrap();
        let c = center(b);
        let from = Frame::new(c, e.plane.normal(), e.plane.x);
        let n = count.max(1);
        let step = if path.closed {
            path.length() / n as f64
        } else if n > 1 {
            path.length() / (n - 1) as f64
        } else {
            0.0
        };
        let mut out = Vec::new();
        for k in 0..n {
            let s = k as f64 * step;
            let to = if orient {
                path.frame(
                    s,
                    if path.closed {
                        None
                    } else {
                        Some(e.plane.normal())
                    },
                )
            } else {
                Frame {
                    origin: path.at(s),
                    ..from
                }
            };
            let m = frame_matrix(&from, &to, 1.0);
            for o in &objects {
                if o.id != id {
                    out.push(ObjectMatrix {
                        id: o.id.clone(),
                        matrix: m,
                    });
                }
            }
        }
        Ok((out, n))
    };
    flow.step(
        e,
        &|c, _| {
            let count = c.n(0).unwrap_or(8.0) as usize;
            match plan(count, c.yes(&flow, "Orient")) {
                Ok((ms, _)) => {
                    let mut transforms: Vec<[f64; 16]> = ms.iter().map(|m| m.matrix).collect();
                    transforms.dedup();
                    Show {
                        transforms,
                        ..Default::default()
                    }
                }
                Err(_) => Show::default(),
            }
        },
        &|c| {
            let count = c.n(0).unwrap().round();
            if count < 1.0 {
                return Err("the number of items must be at least 1".into());
            }
            let (per_object, n) = plan(count as usize, c.yes(&flow, "Orient"))?;
            Ok(Reply::finish_transform(TransformOp {
                matrices: Vec::new(),
                keep_original: true,
                per_object,
            })
            .say(format!("{label}: {n} items along the curve")))
        },
    )
}

fn array_curve_on_surface(e: &Event) -> Reply {
    let flow = Flow::new("Array Along Curve on Surface").number("Number of items", 8.0);
    let plan = |count: usize| -> Result<Vec<ObjectMatrix>, String> {
        let (surface, rest) = split_surface_target(e)
            .ok_or("Array Along Curve on Surface: select the objects, a curve and the surface")?;
        let n = e.plane.normal();
        let target = Target::new(surface.mesh.as_ref().unwrap(), n).ok_or("surface")?;
        let curve = rest
            .iter()
            .find(|s| s.kind == "curve" && s.points.len() >= 2)
            .ok_or("Array Along Curve on Surface: select a curve on the surface")?;
        let path = Path::new(&curve.points, curve.closed).ok_or("curve")?;
        let objects: Vec<&Selected> = rest.iter().copied().filter(|s| s.id != curve.id).collect();
        let b = bbox(&objects).ok_or("Array Along Curve on Surface: select the objects")?;
        let low = e.plane.local(b[0])[2];
        let mid = e.plane.local(center(b));
        let from = Frame::new(e.plane.at(mid[0], mid[1], low), n, e.plane.x);
        let count = count.max(1);
        let step = if path.closed {
            path.length() / count as f64
        } else if count > 1 {
            path.length() / (count - 1) as f64
        } else {
            0.0
        };
        let mut out = Vec::new();
        for k in 0..count {
            let s = k as f64 * step;
            let p = path.at(s);
            let (hit, normal) = target.drop(add(p, scale(n, 1e6))).unwrap_or((p, n));
            let to = Frame::new(hit, normal, path.tangent(s));
            let m = frame_matrix(&from, &to, 1.0);
            out.extend(objects.iter().map(|o| ObjectMatrix {
                id: o.id.clone(),
                matrix: m,
            }));
        }
        Ok(out)
    };
    flow.step(e, &|_, _| Show::default(), &|c| {
        let count = c.n(0).unwrap().round().max(1.0) as usize;
        let per_object = plan(count)?;
        Ok(Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: true,
            per_object,
        })
        .say(format!("Array Along Curve on Surface: {count} items")))
    })
}

fn array_along_surface(e: &Event) -> Reply {
    let flow = Flow::new("Array Along Surface")
        .number("Number in the CPlane x direction", 4.0)
        .number("Number in the CPlane y direction", 4.0);
    flow.step(e, &|_, _| Show::default(), &|c| {
        let (nu, nv) = (
            c.n(0).unwrap().round().max(1.0) as usize,
            c.n(1).unwrap().round().max(1.0) as usize,
        );
        let (surface, objects) = split_surface_target(e)
            .ok_or("Array Along Surface: select the objects and the surface")?;
        if objects.is_empty() {
            return Err("Array Along Surface: select the objects as well as the surface".into());
        }
        let n = e.plane.normal();
        let target = Target::new(surface.mesh.as_ref().unwrap(), n).ok_or("surface")?;
        let b = bbox(&objects).unwrap();
        let mid = e.plane.local(center(b));
        let from = Frame::new(
            e.plane.at(mid[0], mid[1], e.plane.local(b[0])[2]),
            n,
            e.plane.x,
        );
        let (lo, hi) = (
            e.plane.local(surface.bbox[0]),
            e.plane.local(surface.bbox[1]),
        );
        let (x0, x1) = (lo[0].min(hi[0]), lo[0].max(hi[0]));
        let (y0, y1) = (lo[1].min(hi[1]), lo[1].max(hi[1]));
        let mut per_object = Vec::new();
        for i in 0..nu {
            for j in 0..nv {
                let u = x0 + (x1 - x0) * (i as f64 + 0.5) / nu as f64;
                let v = y0 + (y1 - y0) * (j as f64 + 0.5) / nv as f64;
                let Some((hit, normal)) = target.drop(e.plane.at(u, v, 1e6)) else {
                    continue;
                };
                let m = frame_matrix(&from, &Frame::new(hit, normal, e.plane.x), 1.0);
                per_object.extend(objects.iter().map(|o| ObjectMatrix {
                    id: o.id.clone(),
                    matrix: m,
                }));
            }
        }
        if per_object.is_empty() {
            return Err("Array Along Surface: the grid misses the surface".into());
        }
        let items = per_object.len() / objects.len();
        Ok(Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: true,
            per_object,
        })
        .say(format!("Array Along Surface: {items} items")))
    })
}

/// Orient on Surface (Rhino / Matrix): the selected objects stand on the surface, perpendicular
/// to it, at each clicked point. With `Copy=Yes` (the default) every click places another copy
/// and the command keeps going until Enter, so objects can be strewn over the surface; with
/// `Copy=No` the originals move once. `Flip` turns them to the other side of the surface.
fn orient_on_surface(e: &Event) -> Reply {
    const LABEL: &str = "Orient on Surface";
    let n = e.plane.normal();
    let Some((surface, objects)) = split_surface_target(e) else {
        let r = Reply::ask(
            format!("{LABEL}: select the objects and the surface to orient on, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{LABEL}: select the objects and the surface"))
        } else {
            r
        };
    };
    if objects.is_empty() {
        let r = Reply::ask(
            format!("{LABEL}: select the objects to orient as well as the surface, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!(
                "{LABEL}: select the objects as well as the surface"
            ))
        } else {
            r
        };
    }
    let target = Target::new(surface.mesh.as_ref().unwrap(), n);
    let b = bbox(&objects).unwrap();
    let mid = e.plane.local(center(b));
    // The objects' base: the middle of their bottom in the CPlane, standing along its normal.
    let from = Frame::new(
        e.plane.at(mid[0], mid[1], e.plane.local(b[0])[2]),
        n,
        e.plane.x,
    );
    let plan = |p: Vec3, flip: bool| -> Option<[f64; 16]> {
        let (hit, normal) = target.as_ref()?.drop(add(p, scale(n, 1e6)))?;
        let up = if flip { scale(normal, -1.0) } else { normal };
        Some(frame_matrix(&from, &Frame::new(hit, up, e.plane.x), 1.0))
    };
    strew(
        e,
        LABEL,
        &objects,
        &plan,
        "the point is not over the surface",
    )
}

/// Place the objects at clicked points, Rhino style: `Copy=Yes` (default) keeps placing copies
/// until Enter (`Undo` takes the last one back), `Copy=No` moves the originals once; `Flip`
/// turns them over. `plan` gives the transform for a point (and the flip), `None` when the
/// point does not reach the target.
fn strew(
    e: &Event,
    label: &str,
    objects: &[&Selected],
    plan: &dyn Fn(Vec3, bool) -> Option<[f64; 16]>,
    missed: &str,
) -> Reply {
    let st = &e.state;
    let mut copy = st["copy"].as_bool().unwrap_or(true);
    let mut flip = st["flip"].as_bool().unwrap_or(false);
    let mut placed: Vec<Vec3> = serde_json::from_value(st["placed"].clone()).unwrap_or_default();
    let finish = |ms: Vec<[f64; 16]>, copy: bool| -> Reply {
        let count = ms.len();
        let per_object = ms
            .iter()
            .flat_map(|m| {
                objects.iter().map(move |o| ObjectMatrix {
                    id: o.id.clone(),
                    matrix: *m,
                })
            })
            .collect();
        Reply::finish_transform(TransformOp {
            matrices: Vec::new(),
            keep_original: copy,
            per_object,
        })
        .say(if copy {
            format!("{label}: {count} copies placed")
        } else {
            format!("{label}: placed")
        })
    };
    let mut hover = None;
    match &e.input {
        Input::Option { name, value } if name == "Copy" => {
            copy = value
                .as_deref()
                .map_or(!copy, |v| v.eq_ignore_ascii_case("yes"));
        }
        Input::Option { name, .. } if name == "Flip" => flip = !flip,
        Input::Hover { point } => hover = Some(*point),
        Input::Point { point } => match plan(*point, flip) {
            None => {
                return ask_orient(label, copy, flip, &placed, &[], objects)
                    .refuse(format!("{label}: {missed}"));
            }
            Some(m) if !copy => return finish(vec![m], false),
            Some(_) => placed.push(*point),
        },
        Input::Undo => {
            placed.pop();
        }
        Input::Enter => {
            let ms: Vec<_> = placed.iter().filter_map(|p| plan(*p, flip)).collect();
            return if ms.is_empty() {
                Reply::finish(Vec::new()).say(format!("{label}: nothing placed"))
            } else {
                finish(ms, true)
            };
        }
        _ => {}
    }
    let mut shown: Vec<[f64; 16]> = placed.iter().filter_map(|p| plan(*p, flip)).collect();
    shown.extend(hover.and_then(|p| plan(p, flip)));
    ask_orient(label, copy, flip, &placed, &shown, objects)
}

/// The Orient on Surface prompt: options, state and the objects drawn where they would go.
fn ask_orient(
    label: &str,
    copy: bool,
    flip: bool,
    placed: &[Vec3],
    shown: &[[f64; 16]],
    objects: &[&Selected],
) -> Reply {
    let prompt = if copy && !placed.is_empty() {
        format!(
            "{label}: {} placed. Point on the surface for the next copy, Enter when done",
            placed.len()
        )
    } else {
        format!("{label}: point to place the objects")
    };
    let mut options = vec![
        OptionToken::toggle("Copy", copy),
        OptionToken::toggle("Flip", flip),
    ];
    if !placed.is_empty() {
        options.push(OptionToken::action("Undo"));
    }
    let mut preview = Vec::new();
    for m in shown {
        for o in objects {
            preview.extend(moved_preview(o, m));
        }
    }
    Reply::ask(
        prompt,
        json!({ "copy": copy, "flip": flip, "placed": placed }),
    )
    .with_options(options)
    .preview(preview)
}

/// An object drawn under a row-major 4×4 matrix (its mesh, else its curve points).
fn moved_preview(o: &Selected, m: &[f64; 16]) -> Option<Geo> {
    let t = |p: &Vec3| -> Vec3 {
        [
            m[0] * p[0] + m[1] * p[1] + m[2] * p[2] + m[3],
            m[4] * p[0] + m[5] * p[1] + m[6] * p[2] + m[7],
            m[8] * p[0] + m[9] * p[1] + m[10] * p[2] + m[11],
        ]
    };
    if let Some(mesh) = &o.mesh {
        return Some(Geo::Mesh {
            name: o.name.clone(),
            vertices: mesh.vertices.iter().map(t).collect(),
            triangles: mesh.triangles.clone(),
            meta: None,
            layer: None,
        });
    }
    (o.points.len() >= 2).then(|| Geo::Polyline {
        points: o.points.iter().map(t).collect(),
        closed: o.closed,
    })
}

fn orient_perpendicular(e: &Event) -> Reply {
    const LABEL: &str = "Orient Perpendicular to Curve";
    let Some((path, id, objects)) = split_curve_target(e) else {
        let r = Reply::ask(
            format!("{LABEL}: select the objects and the curve, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{LABEL}: select the objects and the curve"))
        } else {
            r
        };
    };
    let objects: Vec<&Selected> = objects.into_iter().filter(|s| s.id != id).collect();
    let Some(b) = bbox(&objects) else {
        let r = Reply::ask(
            format!("{LABEL}: select the objects as well as the curve, Enter"),
            json!({}),
        )
        .want(Want::Objects);
        return if matches!(e.input, Input::Enter) {
            r.refuse(format!("{LABEL}: select the objects as well as the curve"))
        } else {
            r
        };
    };
    let from = Frame::new(center(b), e.plane.normal(), e.plane.x);
    let plan = |p: Vec3, flip: bool| -> Option<[f64; 16]> {
        let s = path.closest(p);
        // The objects' CPlane normal follows the curve direction there.
        let t = path.tangent(s);
        let t = if flip { scale(t, -1.0) } else { t };
        let to = Frame::new(path.at(s), t, cross(t, e.plane.normal()));
        Some(frame_matrix(&from, &to, 1.0))
    };
    strew(e, LABEL, &objects, &plan, "no curve point there")
}

// ---- cages -----------------------------------------------------------------------------------

/// The 8 corners of a box in CPlane coordinates (bottom 4 counter-clockwise, then top 4).
fn box_corners(e: &Event, lo: Vec3, hi: Vec3) -> [Vec3; 8] {
    let p = |x: f64, y: f64, z: f64| e.plane.at(x, y, z);
    [
        p(lo[0], lo[1], lo[2]),
        p(hi[0], lo[1], lo[2]),
        p(hi[0], hi[1], lo[2]),
        p(lo[0], hi[1], lo[2]),
        p(lo[0], lo[1], hi[2]),
        p(hi[0], lo[1], hi[2]),
        p(hi[0], hi[1], hi[2]),
        p(lo[0], hi[1], hi[2]),
    ]
}

fn create_cage(e: &Event) -> Reply {
    if e.selection.is_empty() || !matches!(e.input, Input::Enter) {
        let r = Reply::ask(
            format!(
                "Create Cage: {} objects. Enter creates a box cage around them",
                e.selection.len()
            ),
            json!({}),
        )
        .want(Want::Objects);
        return if e.selection.is_empty() && matches!(e.input, Input::Enter) {
            r.refuse("Create Cage: select the objects to cage")
        } else {
            r
        };
    }
    let objs: Vec<&Selected> = e.selection.iter().collect();
    // The box in CPlane coordinates, a little larger than the objects.
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for s in &objs {
        for x in [s.bbox[0][0], s.bbox[1][0]] {
            for y in [s.bbox[0][1], s.bbox[1][1]] {
                for z in [s.bbox[0][2], s.bbox[1][2]] {
                    let l = e.plane.local([x, y, z]);
                    for i in 0..3 {
                        lo[i] = lo[i].min(l[i]);
                        hi[i] = hi[i].max(l[i]);
                    }
                }
            }
        }
    }
    let pad = 0.02 * dist(lo, hi).max(1.0);
    for i in 0..3 {
        lo[i] -= pad;
        hi[i] += pad;
    }
    let c = box_corners(e, lo, hi);
    let local = mesh::cuboid([0.0; 3], [1.0; 3]);
    // Same vertex order as `box_corners` (cuboid: bottom 0..3, top 4..7).
    let vertices: Vec<Vec3> = (0..8).map(|i| c[i]).collect();
    let record = json!({"om9_cage": {"corners": vertices}});
    let geo = Geo::Mesh {
        name: "Cage".into(),
        vertices,
        triangles: local.triangles,
        meta: Some(record),
        layer: None,
    };
    Reply::finish(vec![geo]).say(
        "Create Cage: move the cage's points (Edit Points), then run Cage Edit with the cage and the objects selected",
    )
}

/// Trilinear map from the original cage corners to the current ones.
pub fn cage_map(p: Vec3, original: &[Vec3; 8], current: &[Vec3; 8]) -> Vec3 {
    let o = original;
    let ex = sub(o[1], o[0]);
    let ey = sub(o[3], o[0]);
    let ez = sub(o[4], o[0]);
    let q = sub(p, o[0]);
    let (u, v, w) = (
        dot(q, ex) / dot(ex, ex),
        dot(q, ey) / dot(ey, ey),
        dot(q, ez) / dot(ez, ez),
    );
    let c = current;
    let lerp = |a: Vec3, b: Vec3, t: f64| add(a, scale(sub(b, a), t));
    let bottom = lerp(lerp(c[0], c[1], u), lerp(c[3], c[2], u), v);
    let top = lerp(lerp(c[4], c[5], u), lerp(c[7], c[6], u), v);
    lerp(bottom, top, w)
}

fn cage_edit(e: &Event) -> Reply {
    let find = || -> Result<([Vec3; 8], [Vec3; 8], &Selected, Vec<&Selected>), String> {
        let cage = e
            .selection
            .iter()
            .find(|s| s.gem.as_ref().is_some_and(|g| g.get("om9_cage").is_some()))
            .ok_or("Cage Edit: select a cage (from Create Cage) and the objects")?;
        let rec = &cage.gem.as_ref().unwrap()["om9_cage"]["corners"];
        let original: Vec<Vec3> = serde_json::from_value(rec.clone())
            .map_err(|_| "Cage Edit: the cage record is damaged")?;
        let current = cage
            .mesh
            .as_ref()
            .ok_or("Cage Edit: the cage has no points")?;
        if original.len() != 8 || current.vertices.len() != 8 {
            return Err("Cage Edit: the cage must keep its 8 points".into());
        }
        let objects: Vec<&Selected> = e.selection.iter().filter(|s| s.id != cage.id).collect();
        if objects.is_empty() {
            return Err("Cage Edit: select the objects as well as the cage".into());
        }
        Ok((
            std::array::from_fn(|i| original[i]),
            std::array::from_fn(|i| current.vertices[i]),
            cage,
            objects,
        ))
    };
    if !matches!(e.input, Input::Enter) {
        let prompt = match find() {
            Ok((_, _, _, objs)) => format!(
                "Cage Edit: {} objects follow the cage. Enter to apply",
                objs.len()
            ),
            Err(w) => w,
        };
        let mut r = Reply::ask(prompt, json!({})).want(Want::Objects);
        if let Ok((o, c, _, objs)) = find() {
            r.preview = show_deform(&only(e, &objs), &|p| cage_map(p, &o, &c)).geo;
        }
        return r;
    }
    match find() {
        Ok((o, c, _, objs)) => {
            finish_deform(&only(e, &objs), "Cage Edit", &|p| cage_map(p, &o, &c)).unwrap_or_else(
                |w| {
                    Reply::ask(
                        "Cage Edit: select the cage and the objects, Enter",
                        json!({}),
                    )
                    .want(Want::Objects)
                    .refuse(w)
                },
            )
        }
        Err(w) => Reply::ask(
            "Cage Edit: select the cage and the objects, Enter",
            json!({}),
        )
        .want(Want::Objects)
        .refuse(w),
    }
}

fn solid_points(e: &Event) -> Reply {
    om9_top11::edit_points(e, true, "Solid Points On")
}

#[cfg(test)]
mod tests;
