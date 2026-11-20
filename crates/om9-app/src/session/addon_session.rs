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
use om9_addon_abi::session::{self as s, Event, Geo, Input, Reply, Selected, SubPick, Want};
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

pub(super) type BooleanDefinitions = std::collections::BTreeMap<om9_core::identity::DefinitionId, u64>;

/// Resolve a direct mesh or a chain of single-member mesh instances in document space.
pub(super) fn boolean_mesh_source<'a>(document: &'a om9_core::model::Document, root: &'a Entity, max_depth: usize)
    -> Result<Option<(&'a Mesh, Transform, BooleanDefinitions)>> {
    let mut entity = root;
    let mut placement = Transform::identity();
    let mut definitions = BooleanDefinitions::new();
    loop {
        placement = placement.compose(entity.placement)?;
        match &entity.geometry {
            Geometry::Mesh(mesh) => return Ok(Some((mesh, placement, definitions))),
            Geometry::Instance(id) => {
                if definitions.len() >= max_depth { return Err(Error::Budget("Boolean instance depth")); }
                let definition = document.definitions.get(id).ok_or(Error::Missing("Boolean instance definition"))?;
                if definitions.insert(*id, definition.revision).is_some() { return Err(Error::Invalid("Boolean instance cycle")); }
                if definition.members.len() != 1 { return Err(Error::Unsupported("Boolean instance requires one mesh member")); }
                entity = &definition.members[0];
            }
            _ if definitions.is_empty() => return Ok(None),
            _ => return Err(Error::Unsupported("Boolean instance requires a mesh member")),
        }
    }
}

/// Build Boolean mesh input from document geometry, never from display buffers.
/// Planar polygon faces use bounded source triangulation; invalid faces refuse.
pub(super) fn boolean_source_mesh(mesh: &Mesh, placement: &Transform, k: f64, tolerance_mm: f64, cancel: Option<&om9_core::transaction::Cancellation>) -> Result<s::SelectedMesh> {
    let check_cancel = || if cancel.is_some_and(om9_core::transaction::Cancellation::is_canceled) { Err(Error::Canceled) } else { Ok(()) };
    check_cancel()?;
    let count = mesh.faces.iter().enumerate().try_fold(0usize, |sum, (index, face)| {
        if index & 255 == 0 { check_cancel()?; }
        if face.len() < 3 { return Err(Error::Invalid("Boolean mesh face")); }
        sum.checked_add(face.len().checked_sub(2).ok_or(Error::Invalid("Boolean mesh face"))?)
            .ok_or(Error::Overflow("Boolean mesh triangles"))
    })?;
    if count > 25_000 { return Err(Error::Budget("Boolean mesh triangles")); }
    let mut vertices = Vec::new();
    vertices.try_reserve_exact(mesh.vertices.len()).map_err(|_| Error::Budget("Boolean mesh vertices"))?;
    for (index, point) in mesh.vertices.iter().enumerate() {
        if index & 255 == 0 { check_cancel()?; }
        let world = mm(placement.point(*point)?, k);
        if world.iter().any(|coordinate| !coordinate.is_finite()) { return Err(Error::Invalid("Boolean mesh world vertex")); }
        vertices.push(world);
    }
    let mut triangles = Vec::new();
    triangles.try_reserve_exact(count).map_err(|_| Error::Budget("Boolean mesh triangles"))?;
    let mut triangulation_work = 0;
    for face in &mesh.faces {
        check_cancel()?;
        if face.len() > 3 {
            let result = if let Some(cancel) = cancel {
                om9_core::mesh_triangulation::triangulate_face_cancellable(&vertices, face, tolerance_mm, &mut triangulation_work, 2_000_000, cancel)
            } else {
                om9_core::mesh_triangulation::triangulate_face(&vertices, face, tolerance_mm, &mut triangulation_work, 2_000_000)
            }?;
            triangles.extend(result);
            continue;
        }
        let point = |index: u32| vertices.get(index as usize).copied().ok_or(Error::Invalid("Boolean mesh face index"));
        let a = point(face[0])?;
        let normal = s::cross(s::sub(point(face[1])?, a), s::sub(point(face[2])?, a));
        let normal_len = s::norm(normal);
        if !normal_len.is_finite() || normal_len == 0.0 { return Err(Error::Invalid("Boolean mesh degenerate face")); }
        for i in 1..face.len() - 1 { triangles.push([face[0], face[i], face[i + 1]]); }
    }
    Ok(s::SelectedMesh { vertices, triangles })
}

#[derive(Debug, Clone)]
pub struct AddonSession {
    prompt_session: uuid::Uuid,
    prompt_revision: u64,
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
    /// The subobject picks seen last (a second edge of the same object is a new selection).
    sub_seen: u64,
    /// The pointer position last sent as a hover event; the UI reports it every frame.
    hover_at: Option<Point3>,
    /// The command reads the triangles of selected meshes (`mesh_data = true`).
    mesh_data: bool,
    /// The command reads trimmed surfaces exactly (`brep_data = true`).
    brep_data: bool,
    /// Every object selected at some step: a command that asks for two groups (Boolean
    /// Difference: first the objects, then the cutters) may change all of them.
    pub(super) seen: std::collections::BTreeSet<om9_core::identity::ObjectId>,
    /// Boolean Builder's accepted first group is retained by its add-on state.
    pub(super) boolean_sources: std::collections::BTreeMap<om9_core::identity::ObjectId, om9_core::identity::SourceStamp>,
    /// Selected inputs used for the last Boolean preview, including the current cutters.
    pub(super) boolean_preview_sources: std::collections::BTreeMap<om9_core::identity::ObjectId, om9_core::identity::SourceStamp>,
    pub(super) boolean_definitions: BooleanDefinitions,
    pub(super) boolean_preview_definitions: BooleanDefinitions,
    pub(super) boolean_source_tolerance: Option<f64>,
    boolean_preview_tolerance: Option<f64>,
    /// Where the last selecting click landed (document units): Fillet, Extend, Match … read
    /// which part of a curve was picked.
    pick: Option<Point3>,
}

impl AddonSession {
    /// Identity of the current advancing input state, excluding ordinary hover previews.
    pub fn prompt_token(&self) -> (uuid::Uuid, u64) {
        (self.prompt_session, self.prompt_revision)
    }
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
        Geo::Trimmed {
            surface,
            loops,
            display,
        } => return trimmed_geometry(surface, loops, display.as_deref(), k),
        Geo::Brep {
            faces,
            solid,
            display,
        } => return brep_geometry(faces, *solid, display.as_deref(), k),
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

/// One add-on face for the kernel: a surface and its trim loops (lines or NURBS lying on
/// it), in document units. A plain surface is a face with no loops (the whole surface).
#[cfg(feature = "occt-brep")]
fn face_parts(g: &Geo, k: f64) -> Result<(om9_core::geometry::NurbsSurface, Vec<Vec<NurbsCurve>>)> {
    let (surface, loops): (&Geo, &[Vec<Geo>]) = match g {
        Geo::Trimmed { surface, loops, .. } => (surface, loops),
        Geo::Surface { .. } => (g, &[]),
        _ => return Err(Error::Invalid("a face is a surface or a trimmed surface")),
    };
    let Geometry::Surface(srf) = to_geometry(surface, k)?.0 else {
        return Err(Error::Invalid("trimmed surface base"));
    };
    let mut rings = Vec::with_capacity(loops.len());
    for ring in loops {
        let mut curves = Vec::with_capacity(ring.len());
        for g in ring {
            match to_geometry(g, k)?.0 {
                Geometry::Curve(Curve::Nurbs(c)) => curves.push(c),
                Geometry::Curve(Curve::Line { start, end, .. }) => curves.push(NurbsCurve {
                    degree: 1,
                    poles: vec![start, end],
                    weights: vec![1.0; 2],
                    knots: vec![0.0, 0.0, 1.0, 1.0],
                    domain: [0.0, 1.0],
                    periodic: false,
                    reversed: false,
                    seam: None,
                }),
                _ => return Err(Error::Invalid("trim loop curves are lines or NURBS")),
            }
        }
        rings.push(curves);
    }
    Ok((srf, rings))
}

#[cfg(feature = "occt-brep")]
thread_local! {
    /// B-Reps made for the objects being committed: their display meshes are staged with the
    /// transaction (the document keeps only the canonical source).
    static NEW_BREPS: std::cell::RefCell<Vec<om9_core::brep::Brep>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The document geometry of a B-Rep made by the kernel; the B-Rep waits in [`NEW_BREPS`].
#[cfg(feature = "occt-brep")]
fn new_brep(brep: om9_core::brep::Brep) -> om9_core::brep::CanonicalBrep {
    use om9_core::brep::BrepSourceAccess;
    let source = brep.to_canonical();
    NEW_BREPS.with(|c| c.borrow_mut().push(brep));
    source
}

/// A trimmed surface from an add-on: an exact face made by the B-Rep kernel (the loops'
/// curves are NURBS lying on the surface). A build without the kernel keeps the add-on's
/// display mesh (or the untrimmed surface).
fn trimmed_geometry(
    surface: &Geo,
    loops: &[Vec<Geo>],
    display: Option<&Geo>,
    k: f64,
) -> Result<(Geometry, Option<Value>, String)> {
    #[cfg(feature = "occt-brep")]
    {
        let _ = display;
        let (srf, rings) = face_parts(
            &Geo::Trimmed { surface: Box::new(surface.clone()), loops: loops.to_vec(), display: None },
            k,
        )?;
        let limits = om9_core::budget::Limits::default();
        let brep = om9_kernel::creation::trimmed_face(
            &srf,
            &rings,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .map_err(|e| Error::Io(format!("trimmed surface: {e}")))?;
        Ok((Geometry::Brep(new_brep(brep)), None, "Surface".into()))
    }
    #[cfg(not(feature = "occt-brep"))]
    {
        let _ = loops;
        to_geometry(display.unwrap_or(surface), k)
    }
}

/// Sewing tolerance for add-on faces (millimetres): Rhino joins edges within twice the
/// absolute tolerance (0.001 mm in jewellery templates); add-on faces are fitted to 0.001 mm.
#[cfg(feature = "occt-brep")]
const SEW_TOLERANCE_MM: f64 = 0.01;

/// A polysurface or solid from an add-on: its faces made by the B-Rep kernel and sewn into one
/// body; `solid` refuses anything but one closed solid. A build without the kernel keeps the
/// add-on's display mesh.
fn brep_geometry(
    faces: &[Geo],
    solid: bool,
    display: Option<&Geo>,
    k: f64,
) -> Result<(Geometry, Option<Value>, String)> {
    #[cfg(feature = "occt-brep")]
    {
        let _ = display;
        let parts = faces.iter().map(|f| face_parts(f, k)).collect::<Result<Vec<_>>>()?;
        let limits = om9_core::budget::Limits::default();
        let brep = om9_kernel::editing::sew_faces(
            &parts,
            solid,
            SEW_TOLERANCE_MM / k,
            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
            &limits,
        )
        .map_err(|e| Error::Io(format!("{}: {e}", if solid { "solid" } else { "polysurface" })))?;
        Ok((Geometry::Brep(new_brep(brep)), None, if solid { "Solid" } else { "Polysurface" }.into()))
    }
    #[cfg(not(feature = "occt-brep"))]
    {
        let _ = (faces, solid);
        match display {
            Some(d) => to_geometry(d, k),
            None => Err(Error::Invalid("this build has no B-Rep kernel and the body has no display mesh")),
        }
    }
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
        // A polysurface or solid previews as its faces' boundaries (and its display mesh).
        Geo::Brep { faces, display, .. } => {
            let mut out: Vec<[Point3; 2]> = faces.iter().flat_map(|f| preview_lines(f, k, limits)).collect();
            if let Some(d) = display {
                out.extend(preview_lines(d, k, limits));
            }
            out
        }
        // A trimmed surface previews as its boundary loops (and its display mesh's edges).
        Geo::Trimmed { loops, display, .. } => {
            let mut out: Vec<[Point3; 2]> = loops
                .iter()
                .flatten()
                .flat_map(|c| preview_lines(c, k, limits))
                .collect();
            if let Some(d) = display {
                out.extend(preview_lines(d, k, limits));
            }
            out
        }
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
        Geo::Trimmed {
            surface,
            loops,
            display,
        } => Geo::Trimmed {
            surface: Box::new(geo_in_local(surface, placement, k)?),
            loops: loops
                .iter()
                .map(|lp| lp.iter().map(|c| geo_in_local(c, placement, k)).collect::<Result<Vec<_>>>())
                .collect::<Result<_>>()?,
            display: display.as_deref().map(|d| geo_in_local(d, placement, k).map(Box::new)).transpose()?,
        },
        Geo::Brep {
            faces,
            solid,
            display,
        } => Geo::Brep {
            faces: faces.iter().map(|f| geo_in_local(f, placement, k)).collect::<Result<_>>()?,
            solid: *solid,
            display: display.as_deref().map(|d| geo_in_local(d, placement, k).map(Box::new)).transpose()?,
        },
    })
}

/// Most faces' NURBS kept for `brep_data` commands (by source hash): a command asks for the
/// same selection at every step and pointer move.
#[cfg(feature = "occt-brep")]
const FACE_CACHE: usize = 32;

/// Most faces a selected polysurface or solid may have to be sent exactly (a larger body is
/// sent as its display mesh only: reading it would hold up every step).
#[cfg(feature = "occt-brep")]
const MAX_SENT_FACES: usize = 256;

/// A B-Rep read for add-ons: its faces as exact NURBS, and whether it is one closed solid.
#[cfg(feature = "occt-brep")]
type ReadBrep = std::sync::Arc<(Vec<om9_kernel::face_nurbs::FaceNurbs>, bool)>;

#[cfg(feature = "occt-brep")]
thread_local! {
    static FACES: std::cell::RefCell<Vec<([u8; 32], ReadBrep)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A trimmed surface or B-Rep exactly, for `brep_data` commands: a surface with Rhino trim
/// loops (the parameter-space polygons, run along the surface) or a one-face B-Rep as
/// [`Geo::Trimmed`] (its underlying NURBS surface and trim loops from the kernel); a
/// polysurface or solid as [`Geo::Brep`] of such faces. A face that is its whole surface goes
/// as the plain [`Geo::Surface`]. `None` for anything else.
fn trimmed_in_world(entity: &Entity, k: f64) -> Option<Geo> {
    let limits = om9_core::budget::Limits::default();
    let placement = &entity.placement;
    let p = |q: Point3| placement.point(q).ok().map(|w| mm(w, k));
    if let Ok(Some(t)) = om9_core::rhino_semantics::trimmed_surface_of(entity, &limits) {
        let surface = exact_in_world(&Geometry::Surface(t.underlying.clone()), placement, k)?;
        // Each straight parameter-space segment is a curve on the surface: sampled so a loop
        // has about 256 points (a dense Rhino polygon keeps its own vertices only).
        let run = |lp: &[[f64; 2]]| -> Option<Geo> {
            let n = lp.len();
            let per = (256 / n.max(1)).clamp(1, 8);
            let mut points = Vec::with_capacity(n * per);
            for i in 0..n {
                let (a, b) = (lp[i], lp[(i + 1) % n]);
                for s in 0..per {
                    let f = s as f64 / per as f64;
                    let q = om9_core::surface_kernel::evaluate(&t.underlying, a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, &limits).ok()?;
                    points.push(p(q)?);
                }
            }
            Some(Geo::Polyline { points, closed: true })
        };
        let mut loops = vec![vec![run(&t.outer)?]];
        for inner in &t.inners {
            loops.push(vec![run(inner)?]);
        }
        return Some(Geo::Trimmed { surface: Box::new(surface), loops, display: None });
    }
    // The display already counts the faces, so a body too large to send is never converted.
    #[cfg(feature = "occt-brep")]
    if let Geometry::Brep(brep) = &entity.geometry
        && (1..=MAX_SENT_FACES).contains(&brep.topology().faces.len())
    {
        let read = read_brep(brep)?;
        let face = |f: &om9_kernel::face_nurbs::FaceNurbs| face_geo(f, &p);
        let (faces, solid) = (&read.0, read.1);
        return match faces.as_slice() {
            [one] => face(one),
            many => Some(Geo::Brep { faces: many.iter().map(face).collect::<Option<_>>()?, solid, display: None }),
        };
    }
    None
}

/// A B-Rep's faces as exact NURBS and whether it is one closed solid, read once per shape
/// (cached by its source hash).
#[cfg(feature = "occt-brep")]
fn read_brep(brep: &om9_core::brep::CanonicalBrep) -> Option<ReadBrep> {
    let limits = om9_core::budget::Limits::default();
    let key = brep.canonical_fingerprints().0;
    FACES.with(|c| c.borrow().iter().find(|(h, _)| *h == key).map(|(_, f)| f.clone())).or_else(|| {
        let faces = om9_kernel::face_nurbs::face_nurbs(brep, &limits).ok()?;
        let solid = faces.len() > 1 && om9_kernel::creation::measure(brep, &limits).is_ok_and(|m| m.solids == 1 && m.closed);
        let f: ReadBrep = std::sync::Arc::new((faces, solid));
        FACES.with(|c| {
            let mut c = c.borrow_mut();
            if c.len() >= FACE_CACHE {
                c.remove(0);
            }
            c.push((key, f.clone()));
        });
        Some(f)
    })
}

/// A kernel NURBS curve in world mm (`p` maps a local point).
#[cfg(feature = "occt-brep")]
fn curve_geo(c: &om9_kernel::face_nurbs::NurbsCurveData, p: &dyn Fn(Point3) -> Option<s::Vec3>) -> Option<Geo> {
    Some(Geo::Nurbs {
        degree: c.degree,
        points: c.poles.iter().map(|q| p(*q)).collect::<Option<_>>()?,
        weights: c.weights.clone(),
        knots: c.knots.clone(),
    })
}

/// A kernel face in world mm: [`Geo::Trimmed`], or the plain surface when it has no loops
/// (it is its whole surface).
#[cfg(feature = "occt-brep")]
fn face_geo(f: &om9_kernel::face_nurbs::FaceNurbs, p: &dyn Fn(Point3) -> Option<s::Vec3>) -> Option<Geo> {
    let surface = Geo::Surface {
        degree: f.degree,
        size: f.size,
        points: f.poles.iter().map(|q| p(*q)).collect::<Option<_>>()?,
        weights: f.weights.clone(),
        knots: f.knots.clone(),
    };
    let loops = f
        .loops
        .iter()
        .map(|lp| lp.iter().map(|c| curve_geo(c, p)).collect::<Option<Vec<Geo>>>())
        .collect::<Option<Vec<_>>>()?;
    Some(if loops.is_empty() { surface } else { Geo::Trimmed { surface: Box::new(surface), loops, display: None } })
}

#[cfg(feature = "occt-brep")]
thread_local! {
    /// Edges read for picked edges, by source hash (most recent last).
    static EDGES: std::cell::RefCell<Vec<([u8; 32], std::sync::Arc<Vec<Option<om9_kernel::face_nurbs::NurbsCurveData>>>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The faces and edges of `entity` picked with Ctrl+Shift+click (`picks`), as the display
/// keys find them: their indices and, with the kernel, their exact geometry in world mm.
fn sub_picks(entity: &Entity, picks: &[&om9_scene::scene::Subelement], k: f64) -> Vec<SubPick> {
    use om9_scene::scene::Subelement;
    let Geometry::Brep(brep) = &entity.geometry else { return Vec::new() };
    let d = brep.topology();
    #[cfg(feature = "occt-brep")]
    let p = |q: Point3| entity.placement.point(q).ok().map(|w| mm(w, k));
    #[cfg(not(feature = "occt-brep"))]
    let _ = k;
    picks
        .iter()
        .filter_map(|pick| match pick {
            Subelement::BrepFace(key) => {
                let i = d.faces.iter().position(|f| f == key)?;
                #[cfg(feature = "occt-brep")]
                let exact = (d.faces.len() <= MAX_SENT_FACES)
                    .then(|| read_brep(brep))
                    .flatten()
                    .and_then(|r| r.0.get(i).and_then(|f| face_geo(f, &p)));
                #[cfg(not(feature = "occt-brep"))]
                let exact = None;
                Some(SubPick { face: Some(i), edge: None, exact })
            }
            Subelement::BrepEdge(key) => {
                let i = d.edges.iter().position(|e| e == key)?;
                #[cfg(feature = "occt-brep")]
                let exact = {
                    let hash = brep.canonical_fingerprints().0;
                    let edges = EDGES.with(|c| c.borrow().iter().find(|(h, _)| *h == hash).map(|(_, e)| e.clone())).or_else(|| {
                        let e = std::sync::Arc::new(om9_kernel::face_nurbs::edge_nurbs(brep, &om9_core::budget::Limits::default()).ok()?);
                        EDGES.with(|c| {
                            let mut c = c.borrow_mut();
                            if c.len() >= FACE_CACHE {
                                c.remove(0);
                            }
                            c.push((hash, e.clone()));
                        });
                        Some(e)
                    });
                    edges.and_then(|e| e.get(i).cloned().flatten()).and_then(|c| curve_geo(&c, &p))
                };
                #[cfg(not(feature = "occt-brep"))]
                let exact = None;
                Some(SubPick { face: None, edge: Some(i), exact })
            }
            _ => None,
        })
        .collect()
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

/// Where each object's pieces sit in the published scene (`Scene::objects`), made once per
/// scene: an add-on step looks up only the picked objects instead of walking the whole
/// document's scene on every click and pointer move.
#[derive(Default)]
pub(super) struct SceneRoots {
    key: Option<(u64, usize, usize)>,
    map: std::collections::HashMap<om9_core::identity::ObjectId, Vec<usize>>,
}

impl Controller {
    /// The scene objects of the selection, in scene order (as a walk over the scene gives them).
    fn selected_scene_objects<'a>(
        &self,
        scene: &'a om9_scene::scene::Scene,
    ) -> Vec<&'a om9_scene::scene::SceneObject> {
        let key = (scene.epoch, scene.objects.as_ptr() as usize, scene.objects.len());
        let mut roots = self.scene_roots.lock().unwrap_or_else(|e| e.into_inner());
        // In pick order (Rhino: sections, profiles, rails are used in the order picked).
        let ordered = self.selected_ordered();
        let lookup = |roots: &SceneRoots| -> Option<Vec<usize>> {
            let mut at = Vec::new();
            for id in &ordered {
                for &i in roots.map.get(id).map_or(&[][..], |v| &v[..]) {
                    // A stale table (same key, other objects) is never trusted.
                    if scene.objects.get(i).is_none_or(|o| o.root != *id) {
                        return None;
                    }
                    at.push(i);
                }
            }
            Some(at)
        };
        let found = if roots.key == Some(key) { lookup(&roots) } else { None };
        let mut at = match found {
            Some(at) => at,
            None => {
                roots.map.clear();
                for (i, object) in scene.objects.iter().enumerate() {
                    roots.map.entry(object.root).or_default().push(i);
                }
                roots.key = Some(key);
                lookup(&roots).unwrap_or_default()
            }
        };
        at.into_iter().map(|i| &scene.objects[i]).collect()
    }

    pub fn addon_session(&self) -> Option<&AddonSession> {
        self.addon_session.as_ref()
    }

    /// Whether the open session is waiting for a viewport click.
    pub fn session_wants_point(&self) -> bool {
        self.addon_session
            .as_ref()
            .is_some_and(|s| matches!(s.reply.want, Want::Point | Want::Number))
    }

    /// A coordinate prompt, as distinct from a numeric prompt that also accepts
    /// viewport input. Nested point aids must not consume number-only input.
    pub fn session_wants_coordinate(&self) -> bool {
        self.addon_session.as_ref().is_some_and(|session| matches!(session.reply.want, Want::Point))
    }

    pub(super) fn boolean_definitions_current(&self, definitions: &BooleanDefinitions) -> bool {
        definitions.iter().all(|(id, revision)| self.document().definitions.get(id).is_some_and(|definition| definition.revision == *revision))
    }

    pub(super) fn invalidate_boolean_preview(&mut self) {
        if let Some(session) = self.addon_session.as_mut() {
            session.lines.clear();
            session.hover_lines.clear();
            session.reply.preview.clear();
            session.selection_seen = self.selection_revision.wrapping_sub(1);
        }
    }

    pub(super) fn accept_boolean_worker_preview(&mut self, mut reply: Reply) -> Result<()> {
        // Preview does not publish the retained exact results.
        reply.replace = Vec::new();
        reply.delete = Vec::new();
        let k = self.document().page_unit.factor();
        let mut lines = Vec::new();
        lines.try_reserve_exact(reply.preview.len()).map_err(|_| Error::Budget("Boolean preview lines"))?;
        for geometry in &reply.preview {
            let Geo::Line { start, end } = geometry else { return Err(Error::Invalid("Boolean preview geometry")); };
            lines.push([doc(*start, k), doc(*end, k)]);
        }
        let session = self.addon_session.as_mut().ok_or(Error::Stale("Boolean preview session"))?;
        session.lines = lines;
        session.hover_lines.clear();
        session.reply.prompt = reply.prompt;
        session.reply.preview = reply.preview;
        session.reply.error = None;
        Ok(())
    }

    fn boolean_selection_definitions(&self) -> Result<BooleanDefinitions> {
        let mut all = BooleanDefinitions::new();
        for id in &self.selection {
            let entity = self.document().entities.get(id).ok_or(Error::Missing("Boolean source"))?;
            if let Some((_, _, definitions)) = boolean_mesh_source(self.document(), entity, self.coordinator.limits().max_depth)? {
                all.extend(definitions);
            }
        }
        Ok(all)
    }

    /// The selection as add-ons see it (world millimetres).
    fn session_selection(&self, k: f64, mesh_data: bool, brep_data: bool, boolean_source: bool) -> Result<Vec<Selected>> {
        use om9_scene::scene::RenderBuffers;
        let mut out: Vec<Selected> = Vec::new();
        let mut triangles_sent = 0usize;
        // Nothing picked: nothing to walk (the scene can hold a whole large file).
        if self.selection.is_empty() {
            return Ok(out);
        }
        let Some(scene) = self.scene.as_ref() else {
            return Ok(out);
        };
        let mut index = std::collections::BTreeMap::new();
        for object in self.selected_scene_objects(scene) {
            if out.len() >= MAX_SELECTION {
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
                    exact: if brep_data {
                        trimmed_in_world(entity, k).or_else(|| exact_in_world(&entity.geometry, &entity.placement, k))
                    } else {
                        exact_in_world(&entity.geometry, &entity.placement, k)
                    },
                    sub: if brep_data {
                        let picks: Vec<&om9_scene::scene::Subelement> = self
                            .subselection()
                            .iter()
                            .filter(|r| r.root == object.root)
                            .map(|r| &r.subset)
                            .collect();
                        sub_picks(entity, &picks, k)
                    } else {
                        Vec::new()
                    },
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
            if mesh_data && !boolean_source
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
        if boolean_source {
            for (id, slot) in &index {
                let sel = &mut out[*slot];
                let entity = self.document().entities.get(id).ok_or(Error::Missing("Boolean source"))?;
                let Some((mesh, placement, _)) = boolean_mesh_source(self.document(), entity, self.coordinator.limits().max_depth)? else { continue };
                let source = boolean_source_mesh(mesh, &placement, k, self.document().effective_tolerance()?.absolute_mm, None)?;
                sel.kind = "mesh".into();
                triangles_sent = triangles_sent.checked_add(source.triangles.len()).ok_or(Error::Overflow("Boolean selection triangles"))?;
                if triangles_sent > MAX_SELECTION_TRIANGLES { return Err(Error::Budget("Boolean selection triangles")); }
                sel.bbox = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
                for p in &source.vertices { for axis in 0..3 {
                    sel.bbox[0][axis] = sel.bbox[0][axis].min(p[axis]);
                    sel.bbox[1][axis] = sel.bbox[1][axis].max(p[axis]);
                }}
                sel.mesh = Some(source);
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
        Ok(out)
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
        for object in self.selected_scene_objects(scene) {
            if lines.len() > 200_000 {
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
        let brep_data = self
            .addons
            .as_ref()
            .and_then(|h| h.lock().ok()?.command_info(code))
            .is_some_and(|i| i.command.brep_data);
        self.cancel();
        self.addon_session = Some(AddonSession {
            prompt_session: uuid::Uuid::new_v4(),
            prompt_revision: 0,
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
            sub_seen: self.sub_revision(),
            hover_at: None,
            mesh_data,
            brep_data,
            seen: Default::default(),
            boolean_sources: Default::default(),
            boolean_preview_sources: Default::default(),
            boolean_definitions: Default::default(),
            boolean_preview_definitions: Default::default(),
            boolean_source_tolerance: None,
            boolean_preview_tolerance: None,
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
        if self.boolean_builder_worker_pending() { return Err(Error::Busy); }
        if matches!(input, Input::Enter) && self.request_ring_rail_confirmation()? { return Ok(()); }
        let Some(session) = self.addon_session.as_ref() else {
            return Err(Error::Missing("interactive command"));
        };
        if session.boolean_sources.values().any(|stamp| self.document().resolve(stamp).is_err())
            || !self.boolean_definitions_current(&session.boolean_definitions)
            || session.boolean_source_tolerance.is_some_and(|t| !self.document().effective_tolerance().is_ok_and(|current| current.absolute_mm == t)) {
            if let Some(session) = self.addon_session.as_mut() {
                session.lines.clear();
                session.hover_lines.clear();
            }
            return Err(Error::Stale("Boolean Builder source; restart command"));
        }
        let capture_boolean_sources = session.code == "BuilderBooleanBuilder"
            && session.boolean_sources.is_empty() && matches!(input, Input::Enter);
        let boolean_tolerance = if session.code == "BuilderBooleanBuilder" {
            Some(self.document().effective_tolerance()?.absolute_mm)
        } else { None };
        let candidate_sources = if capture_boolean_sources {
            self.selection.iter().map(|id| self.document().stamp(*id).map(|stamp| (*id, stamp)))
                .collect::<Result<Vec<_>>>()?
        } else { Vec::new() };
        let k = self.document().page_unit.factor();
        let hover = matches!(input, Input::Hover { .. });
        let plane = self.input_plane();
        let mut settings: std::collections::BTreeMap<String, Value> = self
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
        if session.code == "BuilderBooleanBuilder" {
            settings.insert("_om9_document_tolerance_mm".into(), Value::from(boolean_tolerance.expect("Boolean tolerance")));
        }
        if matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail")
            && matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11")
            && let Some(shape) = &self.ring_rail_shape {
                settings.insert("_om9_ring_rail_shape".into(), serde_json::to_value(shape).map_err(|_| Error::Invalid("custom Ring Rail shape"))?);
            }
        let current: Vec<_> = self.selection.iter().copied().collect();
        let preview_definitions = if session.code == "BuilderBooleanBuilder" {
            Some(self.boolean_selection_definitions()?)
        } else { None };
        let accepted_definitions = if capture_boolean_sources { preview_definitions.clone() } else { None };
        let preview_sources = if session.code == "BuilderBooleanBuilder" {
            Some(current.iter().map(|id| self.document().stamp(*id).map(|stamp| (*id, stamp)))
                .collect::<Result<std::collections::BTreeMap<_, _>>>()?)
        } else { None };
        if let Some(s) = self.addon_session.as_mut() {
            s.seen.extend(current);
            if let Some(stamps) = preview_sources { s.boolean_preview_sources = stamps; }
            if let Some(definitions) = preview_definitions { s.boolean_preview_definitions = definitions; }
            s.boolean_preview_tolerance = boolean_tolerance;
        }
        let Some(session) = self.addon_session.as_ref() else {
            return Err(Error::Missing("interactive command"));
        };
        let worker_roots: Vec<_> = if self.background_boolean_builder && matches!(input, Input::Enter | Input::Selection)
            && matches!(session.owner.as_str(), "om9_matrix" | "om9_boolean_builder")
            && session.code == "BuilderBooleanBuilder"
            && session.reply.state.get("objects").and_then(Value::as_array).is_some_and(|objects| !objects.is_empty()) {
                self.scene.as_ref().into_iter().flat_map(|scene| &scene.objects)
                    .filter(|object| self.selection.contains(&object.root))
                    .map(|object| object.root).collect::<std::collections::BTreeSet<_>>().into_iter().take(MAX_SELECTION).collect()
            } else { Vec::new() };
        let worker = session.reply.state.get("objects").and_then(Value::as_array).is_some_and(|objects|
            worker_roots.iter().any(|id| !objects.iter().any(|object| object.get(0).and_then(Value::as_str) == Some(id.to_string().as_str()))));
        let event = Event {
            command: session.code.clone(),
            input,
            state: session.reply.state.clone(),
            plane: s::Plane {
                origin: mm(plane.origin, k),
                x: plane.x,
                y: plane.y,
            },
            selection: if worker { Vec::new() } else { self.session_selection(k, session.mesh_data, session.brep_data, session.code == "BuilderBooleanBuilder")? },
            pick: session.pick.map(|p| mm(p, k)),
            settings,
        };
        if worker {
                let preview = matches!(event.input, Input::Selection);
                return self.request_boolean_builder_worker(event, preview, worker_roots, k);
            }
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
        if event.settings.contains_key("_om9_ring_rail_shape") && reply.error.is_none()
            && reply.state.get("_om9_ring_rail_curve").is_none() {
                return Err(Error::Unsupported("update the Ring Rail add-on to use custom shapes"));
            }
        if capture_boolean_sources && reply.error.is_none() && !reply.done
            && let Some(objects) = reply.state.get("objects").and_then(Value::as_array)
            && let Some(session) = self.addon_session.as_mut() {
                if let Some(definitions) = accepted_definitions { session.boolean_definitions = definitions; }
                session.boolean_source_tolerance = boolean_tolerance;
                for (id, stamp) in candidate_sources {
                    if objects.iter().any(|entry| entry.get(0).and_then(Value::as_str) == Some(id.to_string().as_str())) {
                        session.boolean_sources.insert(id, stamp);
                    }
                }
            }
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
                    if session.reply.state != reply.state {
                        session.prompt_revision = session.prompt_revision.wrapping_add(1);
                    }
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
            let canonical_ring = matches!(code.as_str(), "BuilderRingRail" | "TopIconRingRail")
                && self.addon_session.as_ref().is_some_and(|session| matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11"));
            let replacing_ring = canonical_ring && self.ring_rail_replacing();
            let before_revision = self.document().revision;
            if !canonical_ring { self.addon_session = None; }
            self.addon_last = Some(code.clone());
            let created = self.commit_session_result(&code, &label, &reply, &seen)?;
            self.addon_session = None;
            // The command's panel closes with it, however it was finished (Enter in the
            // command line, right-click, Accept); its values become the next defaults.
            if let Some(b) = self
                .addon_builder
                .take_if(|b| b.interactive && b.code == code)
            {
                self.store_builder_values(&b);
            }
            if self.join_history_worker_pending() {
                self.events.emit(crate::command_events::EventKind::Progress, &code, "updating History", None);
            } else {
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    &code,
                    &if replacing_ring { if before_revision == self.document().revision { "Ring Rail unchanged".into() } else { "Ring Rail replaced".into() } } else { format!("created {created} objects") },
                    Some(crate::command_events::EventOutcome::Committed),
                );
            }
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
            session.prompt_revision = session.prompt_revision.wrapping_add(1);
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

    pub(crate) fn clear_session_hover(&mut self) {
        if let Some(session) = self.addon_session.as_mut() {
            session.hover_at = None;
            session.hover_lines.clear();
        }
    }

    /// The add-on command right-click or an empty Enter would run again.
    pub fn addon_repeatable(&self) -> Option<&str> {
        self.addon_last.as_deref()
    }

    /// Run the last add-on command again (right-click or Enter with nothing running).
    pub fn repeat_addon_command(&mut self) -> Result<bool> {
        if self.is_running() || self.addon_session.is_some() {
            return Ok(false);
        }
        let Some(code) = self.addon_last.clone() else {
            return Ok(false);
        };
        self.run_addon_command(&code)?;
        Ok(true)
    }

    /// An add-on command is waiting for objects: like Rhino's object prompts, every click or
    /// window adds to what was picked before, instead of replacing it (the command reads the
    /// earlier picks from the selection).
    pub fn addon_collects_objects(&self) -> bool {
        self.addon_session
            .as_ref()
            .is_some_and(|s| matches!(s.reply.want, Want::Objects))
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
        if self.boolean_builder_worker_pending() { return; }
        let Some(s) = self.addon_session.as_ref() else {
            return;
        };
        if s.boolean_sources.values().any(|stamp| self.document().resolve(stamp).is_err())
            || !self.boolean_definitions_current(&s.boolean_definitions)
            || s.boolean_source_tolerance.is_some_and(|t| !self.document().effective_tolerance().is_ok_and(|current| current.absolute_mm == t)) {
            if let Some(s) = self.addon_session.as_mut() {
                s.lines.clear();
                s.hover_lines.clear();
            }
            return;
        }
        let changed_source = s.boolean_preview_sources.values().any(|stamp| self.document().resolve(stamp).is_err())
            || !self.boolean_definitions_current(&s.boolean_preview_definitions)
            || s.boolean_preview_tolerance.is_some_and(|t| !self.document().effective_tolerance().is_ok_and(|current| current.absolute_mm == t));
        let sub = if s.brep_data { self.sub_revision() } else { s.sub_seen };
        if s.selection_seen != self.selection_revision || s.sub_seen != sub || changed_source {
            let rev = self.selection_revision;
            if let Some(s) = self.addon_session.as_mut() {
                s.selection_seen = rev;
                s.sub_seen = sub;
                if s.code == "BuilderBooleanBuilder" {
                    s.lines.clear();
                    s.hover_lines.clear();
                }
            }
            if matches!(self.session_step(Input::Selection), Err(Error::Busy)) && self.boolean_builder_worker_retiring() {
                self.invalidate_boolean_preview();
            }
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
        if self.addon_session.as_ref().is_some_and(|session| matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail")) { self.ring_rail_replace = None; }
        self.cancel_boolean_builder_worker();
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
    /// Official Ring Rail add-ons adapt the canonical builder publication, including
    /// its recipe. F6 can therefore edit either entry through the direct owner.
    fn commit_addon_ring_rail(&mut self, code: &str, reply: &Reply) -> Result<usize> {
        use om9_core::{builder::{BuilderKind, BuilderSession}, request::Value as Parameter, units::LengthMm};
        use om9_addons::settings::SettingValue;
        let custom = self.ring_rail_shape.clone();
        if reply.create.len() != usize::from(custom.is_none()) || !reply.replace.is_empty() || !reply.delete.is_empty()
            || !reply.records.is_empty() || reply.transform.is_some() { return Err(Error::Invalid("Ring Rail reply")); }
        if custom.is_none() && !matches!(&reply.create[0], Geo::Circle { .. }) { return Err(Error::Invalid("Ring Rail circular source")); }
        let (system, size, center) = {
            let builder = self.addon_builder.as_ref().ok_or(Error::Missing("Ring Rail builder"))?;
            let prefix = if code == "TopIconRingRail" { "t11" } else { "rr" };
            let Some(SettingValue::Str(system)) = builder.value(&format!("{prefix}_size_system")) else { return Err(Error::Invalid("Ring Rail size system")) };
            let Some(SettingValue::Float(size)) = builder.value(&format!("{prefix}_ring_size")) else { return Err(Error::Invalid("Ring Rail size")) };
            let coordinate = |name| match builder.value(name) {
                Some(SettingValue::Float(value)) => Ok(*value), None => Ok(0.), _ => Err(Error::Invalid("Ring Rail center")),
            };
            (system.clone(), *size, [coordinate("rr_center_x")?, coordinate("rr_center_y")?, coordinate("rr_center_z")?])
        };
        let diameter = om9_core::ring_size::rail_diameter_mm(&system, size)?;
        let standard = match system.as_str() { "us" => "US", "vnm" => "VNM", "diameter" => "explicit", other => other };
        let mut session = BuilderSession::new(BuilderKind::RingRail, self.document(), &[])?;
        session.set_rail_shape(custom.clone(), self.coordinator.limits())?;
        session.set_field("diameter_mm", Parameter::length(LengthMm::try_new(diameter)?))?;
        session.set_field("standard", Parameter::Choice(standard.into()))?;
        session.set_field("us_size", Parameter::ratio(size)?)?;
        for (name, coordinate) in ["center_x_mm", "center_y_mm", "center_z_mm"].into_iter().zip(center) {
            session.set_field(name, Parameter::length(LengthMm::try_new(coordinate)?))?;
        }
        session.refresh(self.document(), &self.builder_registry()?)?;
        if let Some(shape) = custom {
            let curve: Curve = serde_json::from_value(reply.state.get("_om9_ring_rail_curve").cloned()
                .ok_or(Error::Missing("custom Ring Rail source"))?).map_err(|_| Error::Invalid("custom Ring Rail source"))?;
            curve.validate(self.coordinator.limits())?;
            if curve != shape.sized_curve(diameter, center, "front", 1., self.coordinator.limits())? {
                return Err(Error::Invalid("custom Ring Rail source mismatch"));
            }
        } else {
            let (geometry, _, _) = to_geometry(&reply.create[0], self.document().page_unit.factor())?;
            if session.preview().is_none_or(|preview| preview.geometry != geometry) { return Err(Error::Invalid("Ring Rail source mismatch")); }
        }
        self.builder = Some(session);
        let result = self.builder_apply();
        if result.is_err() { self.builder = None; }
        result.map(|_| 1)
    }

    pub(super) fn commit_session_result(
        &mut self,
        code: &str,
        label: &str,
        reply: &Reply,
        seen: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
    ) -> Result<usize> {
        if matches!(code, "BuilderRingRail" | "TopIconRingRail")
            && self.addon_session.as_ref().is_some_and(|session| matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11")) {
            return self.commit_addon_ring_rail(code, reply);
        }
        let k = self.document().page_unit.factor();
        let layer = self.document().current_layer;
        #[cfg(feature = "occt-brep")]
        NEW_BREPS.with(|c| c.borrow_mut().clear());
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
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("object"))?;
            // The object keeps its placement: the world geometry goes into its own frame.
            let local = geo_in_local(&r.geo, &entity.placement, k)?;
            let (geometry, _, _) = to_geometry(&local, k)?;
            let source = self.document().stamp(id)?;
            // Trim and extrusion records describe the old geometry: the replacement carries its
            // own shape (a trimmed result is a B-Rep face), so they go (Untrim keeps the
            // surface and drops its trims).
            let stale = [
                om9_core::model::TRIM_SURFACE_EXTENSION_SLOT,
                om9_core::model::EXTRUSION_EXTENSION_SLOT,
            ];
            let extensions = stale
                .iter()
                .any(|slot| entity.extensions.contains_key(*slot))
                .then(|| {
                    let mut x = entity.extensions.clone();
                    x.retain(|slot, _| !stale.contains(&slot.as_str()));
                    x
                });
            mutations.push(Mutation::Geometry { source, geometry });
            if let Some(extensions) = extensions {
                mutations.push(Mutation::Extensions { source, extensions });
            }
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
        let undo_label = crate::icons::icon_name(code).unwrap_or(label).to_owned();
        let touches_history = mutations.iter().any(|mutation| match mutation {
            Mutation::Geometry { source, .. } | Mutation::Placement { source, .. } | Mutation::Delete { source } =>
                self.source_edits_update_join_history(std::slice::from_ref(&source.object)),
            _ => false,
        });
        if touches_history {
            if !self.source_mutations_update_join_history(&mutations) {
                return Err(Error::Unsupported("add-on History edit includes unsupported mutations"));
            }
            let (completion, owner) = self.reserve_addon_history_completion(code)?;
            self.request_join_history_edit_worker(TransactionRequest::new(&undo_label, mutations))?;
            self.mark_addon_history_completion(completion, owner);
            return Ok(count);
        }
        let mut held = self.scene_cache_usage()?;
        // One new object (Line, Circle …): add it to the scene incrementally, as OM9's own
        // commands do, instead of rebuilding the scene of the whole document.
        #[cfg(feature = "occt-brep")]
        let mut breps: Vec<om9_core::brep::Brep> = NEW_BREPS.with(|c| std::mem::take(&mut *c.borrow_mut()));
        #[cfg(not(feature = "occt-brep"))]
        let breps: Vec<()> = Vec::new();
        if let ([Mutation::Insert { entity, .. }], true) = (mutations.as_slice(), breps.is_empty()) {
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
        #[allow(unused_mut)]
        let mut prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(&undo_label, mutations), held)?;
        // The kernel's B-Reps bring their display meshes with the transaction.
        #[cfg(feature = "occt-brep")]
        if !breps.is_empty() {
            self.stage_builder_preview_representations(&mut prepared, &mut breps, held)?;
        }
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
