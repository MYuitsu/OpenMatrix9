// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::camera::{Camera, Viewport};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Geometry, Point3, Transform},
    identity::*,
    model::{Document, Entity, Permissions},
    transaction::TransactionCoordinator,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};
mod admission;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderBuffers {
    Points {
        points: Vec<Point3>,
        native_indices: Vec<u32>,
    },
    Curve {
        points: Vec<Point3>,
        /// Native parameters for evaluator-backed curves. Empty for exact
        /// line polylines, which have no global native parameter binding.
        parameters: Vec<f64>,
    },
    Mesh {
        vertices: Vec<Point3>,
        triangles: Vec<([u32; 3], u32)>,
        edges: Vec<[u32; 2]>,
    },
}
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub leaf: ObjectId,
    pub source: SourceStamp,
    pub transform: Transform,
    pub preview: Transform,
    pub color: [u8; 4],
    pub buffers: Arc<RenderBuffers>,
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub reference_snappable: bool,
    pub reference_unlocked: bool,
}
impl SceneObject {
    pub fn has_pick_geometry(&self) -> bool {
        match self.buffers.as_ref() {
            RenderBuffers::Points { points, .. } => !points.is_empty(),
            RenderBuffers::Curve { points, .. } => points.len() >= 2,
            RenderBuffers::Mesh {
                triangles, edges, ..
            } => !triangles.is_empty() || !edges.is_empty(),
        }
    }
    pub fn display_transform(&self) -> Result<Transform> {
        self.preview.compose(self.transform)
    }

    /// World-space bounding sphere of this occurrence, for the CPU picking BVH. Points, curves and
    /// meshes all use their display vertices; the exact geometry test still happens after the BVH.
    pub fn sphere_bounds(&self) -> Result<([f64; 3], f64)> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut include = |p: [f64; 3]| {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        };
        match self.buffers.as_ref() {
            RenderBuffers::Points { points, .. } => {
                for p in points {
                    include(*p);
                }
            }
            RenderBuffers::Curve { points, .. } => {
                for p in points {
                    include(*p);
                }
            }
            RenderBuffers::Mesh { vertices, .. } => {
                for p in vertices {
                    include(*p);
                }
            }
        }
        if !min[0].is_finite() || min[0] > max[0] {
            return Ok(([0.0; 3], 0.0));
        }
        let centre = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let transform = self.display_transform()?;
        let world = transform.point(centre)?;
        // A half longest side is not an enclosing sphere: it rejects box corners. Transform all
        // eight AABB corners so nonuniform scale, shear and previews remain conservative as well.
        let mut radius = 0.0f64;
        for corner in 0..8 {
            let p = transform.point(std::array::from_fn(|axis| {
                if corner & (1 << axis) == 0 {
                    min[axis]
                } else {
                    max[axis]
                }
            }))?;
            radius = radius.max(
                (0..3)
                    .map(|axis| (p[axis] - world[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt(),
            );
        }
        radius += (0..3).map(|axis| world[axis].abs()).fold(1.0, f64::max) * f64::EPSILON * 8.;
        Ok((world, radius))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickIntent {
    Select,
    Reference { snap_locked: bool },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Subelement {
    BrepFace(uuid::Uuid),
    BrepEdge(uuid::Uuid),
    BrepVertex(uuid::Uuid),
    Whole,
    CurveParameter(f64),
    MeshFace(u32),
    MeshEdge { a: u32, b: u32 },
    MeshVertex(u32),
    CloudPoint(u32),
}

/// What kind of subobject a pick may return: the object itself, a vertex, an edge or a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    Object,
    Vertex,
    Edge,
    Face,
}
#[derive(Debug, Clone)]
pub struct PickHit {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub subset: Subelement,
    pub world_point: Point3,
    pub source: SourceStamp,
    pub world_epoch: u64,
    pub screen_distance: f64,
    pub depth: f64,
    pub(crate) camera: Camera,
    pub(crate) viewport: Viewport,
}
impl PickHit {
    pub fn revalidate(&self, c: &TransactionCoordinator, camera: &Camera) -> Result<()> {
        if c.epoch() != self.world_epoch || camera != &self.camera {
            return Err(Error::Stale("pick camera/publication"));
        }
        let d = c
            .world()
            .documents
            .get(&self.source.document)
            .ok_or(Error::Missing("pick document"))?;
        d.resolve(&self.source)?;
        Ok(())
    }
    pub fn revalidate_in_view(
        &self,
        c: &TransactionCoordinator,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<()> {
        self.revalidate(c, camera)?;
        if viewport != self.viewport {
            return Err(Error::Stale("pick viewport rectangle"));
        }
        Ok(())
    }
}
pub struct Scene {
    pub document: Arc<Document>,
    pub epoch: u64,
    pub objects: Vec<SceneObject>,
    pub diagnostics: Vec<String>,
    pub(crate) limits: Limits,
    buffers_usage: Usage,
    /// Picking BVH, built on first use from each occurrence sphere.
    spatial: OnceLock<crate::spatial::SpatialIndex>,
    spheres: OnceLock<Vec<([f64; 3], f64)>>,
}
fn combined(a: Permissions, b: Permissions) -> Permissions {
    Permissions {
        visible: a.visible && b.visible,
        selectable: a.selectable && b.selectable,
        editable: a.editable && b.editable,
        reference_snappable: a.reference_snappable && b.reference_snappable,
    }
}
fn buffer_usage(buffer: &RenderBuffers) -> Result<Usage> {
    let mut usage = Usage {
        bytes: std::mem::size_of::<RenderBuffers>() as u64 + 16,
        ..Usage::default()
    };
    let l = Limits {
        max_bytes: u64::MAX,
        max_points: u64::MAX,
        ..Limits::default()
    };
    match buffer {
        RenderBuffers::Points {
            points,
            native_indices,
        } => {
            usage.add(Usage::array::<Point3>(points.capacity())?, &l)?;
            usage.add(Usage::array::<u32>(native_indices.capacity())?, &l)?;
            usage.points = points.len() as u64;
        }
        RenderBuffers::Curve { points, parameters } => {
            usage.add(Usage::array::<Point3>(points.capacity())?, &l)?;
            usage.add(Usage::array::<f64>(parameters.capacity())?, &l)?;
            usage.points = points.len() as u64;
        }
        RenderBuffers::Mesh {
            vertices,
            triangles,
            edges,
        } => {
            usage.add(Usage::array::<Point3>(vertices.capacity())?, &l)?;
            usage.add(Usage::array::<([u32; 3], u32)>(triangles.capacity())?, &l)?;
            usage.add(Usage::array::<[u32; 2]>(edges.capacity())?, &l)?;
            usage.points = vertices.len() as u64;
        }
    }
    Ok(usage)
}
fn metadata_leaf<'a>(
    document: &'a Document,
    members: &BTreeMap<(DefinitionId, ObjectId), &'a Entity>,
    root: ObjectId,
    path: &[ObjectId],
) -> Result<(&'a Entity, Permissions, bool)> {
    let mut entity = document
        .entities
        .get(&root)
        .ok_or(Error::Missing("scene root"))?;
    let mut permissions = document.permissions_for_entity(entity, true)?;
    let mut unlocked = document
        .permissions_for_entity(entity, false)?
        .reference_snappable;
    for id in path {
        let Geometry::Instance(definition) = entity.geometry else {
            return Err(Error::Stale("scene path"));
        };
        entity = members
            .get(&(definition, *id))
            .copied()
            .ok_or(Error::Stale("scene member"))?;
        permissions = combined(permissions, document.permissions_for_entity(entity, true)?);
        unlocked &= document
            .permissions_for_entity(entity, false)?
            .reference_snappable;
    }
    Ok((entity, permissions, unlocked))
}
/// The initial SDK's polyline is a connected, unreversed list of direct lines.
/// Keep this admission check shared by the allocation plan and display cache;
/// connecting arbitrary polycurve segments would invent geometry between gaps.
fn connected_polyline_segments(curve: &Curve) -> Result<&[Curve]> {
    let Curve::Polycurve {
        segments,
        reversed: false,
    } = curve
    else {
        return Err(Error::Unsupported(
            "polycurve display requires unreversed line segments",
        ));
    };
    if segments.is_empty() {
        return Err(Error::Unsupported(
            "polycurve display requires line segments",
        ));
    }
    let mut previous_end = None;
    for segment in segments {
        let Curve::Line { start, end, .. } = segment else {
            return Err(Error::Unsupported(
                "polycurve display requires direct line segments",
            ));
        };
        if previous_end.is_some_and(|previous| previous != *start) {
            return Err(Error::Unsupported(
                "polycurve display requires connected line segments",
            ));
        }
        previous_end = Some(*end);
    }
    Ok(segments)
}

fn cache(g: &Geometry, usage: &mut Usage, l: &Limits) -> Result<RenderBuffers> {
    match g {
        Geometry::Point(p) => {
            usage.add(
                Usage {
                    points: 1,
                    ..Usage::array::<Point3>(1)?
                },
                l,
            )?;
            Ok(RenderBuffers::Points {
                points: vec![*p],
                native_indices: vec![0],
            })
        }
        Geometry::Cloud(c) => {
            let count = c.points.len();
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<(Point3, u32)>(count)?
                },
                l,
            )?;
            let visible = c
                .points
                .iter()
                .enumerate()
                .filter(|(i, _)| !c.hidden.get(*i).copied().unwrap_or(false))
                .count();
            let mut points = Vec::new();
            let mut native_indices = Vec::new();
            points
                .try_reserve_exact(visible)
                .map_err(|_| Error::Budget("cloud display allocation"))?;
            native_indices
                .try_reserve_exact(visible)
                .map_err(|_| Error::Budget("cloud index allocation"))?;
            for (i, p) in c.points.iter().enumerate() {
                if c.hidden.get(i).copied().unwrap_or(false) {
                    continue;
                }
                points.push(*p);
                native_indices
                    .push(u32::try_from(i).map_err(|_| Error::Overflow("cloud native index"))?);
            }
            Ok(RenderBuffers::Points {
                points,
                native_indices,
            })
        }
        Geometry::Curve(curve @ Curve::Polycurve { .. }) => {
            let segments = connected_polyline_segments(curve)?;
            let count = segments
                .len()
                .checked_add(1)
                .ok_or(Error::Overflow("polyline display vertices"))?;
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<Point3>(count)?
                },
                l,
            )?;
            let mut points = Vec::new();
            points
                .try_reserve_exact(count)
                .map_err(|_| Error::Budget("polyline display allocation"))?;
            for segment in segments {
                let Curve::Line { start, end, .. } = segment else {
                    unreachable!()
                };
                if points.is_empty() {
                    points.push(*start);
                }
                points.push(*end);
            }
            // Exact corners, not a uniform sampling approximation. A polycurve
            // has no global native domain; picking returns whole-object identity.
            Ok(RenderBuffers::Curve {
                points,
                parameters: Vec::new(),
            })
        }
        Geometry::Curve(c) => {
            let (domain, segments) = match c {
                Curve::Line { domain, .. } => (*domain, 1usize),
                Curve::Circle {
                    domain,
                    radius,
                    angle_domain,
                    ..
                } => {
                    let chord = 0.005;
                    let angle = 2. * (1. - (chord / radius).min(1.)).acos();
                    let angles = angle_domain.unwrap_or(*domain);
                    let estimate = ((angles[1] - angles[0]) / angle).ceil().max(16.);
                    if !estimate.is_finite() || estimate > 65535. {
                        return Err(Error::Budget("circle display chord samples"));
                    }
                    (*domain, estimate as usize)
                }
                Curve::Nurbs(c) if !c.periodic => (c.domain, 256),
                _ => return Err(Error::Unsupported("curve display family/segment binding")),
            };
            let count = segments
                .checked_add(1)
                .ok_or(Error::Overflow("display samples"))?;
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<(Point3, f64)>(count)?
                },
                l,
            )?;
            let mut points = Vec::with_capacity(count);
            let mut parameters = Vec::with_capacity(count);
            let evaluator = CurveEvaluator::capture(c, l)?;
            for i in 0..count {
                let fraction = i as f64 / segments as f64;
                let t = if i == segments {
                    domain[1]
                } else {
                    domain[0] + (domain[1] - domain[0]) * fraction
                };
                if matches!(
                    c,
                    Curve::Circle {
                        angle_domain: Some(_),
                        ..
                    }
                ) {
                    let recovered = (t - domain[0]) / (domain[1] - domain[0]);
                    // Native f64 bindings must reproduce the displayed phase.
                    // A valid but unresolvable durable map stays owned, with a
                    // visible diagnostic instead of a collapsed display proxy.
                    if parameters.last().is_some_and(|previous| t <= *previous)
                        || !recovered.is_finite()
                        || (recovered - fraction).abs() > 64. * f64::EPSILON
                    {
                        return Err(Error::Unsupported(
                            "circle display native parameter precision",
                        ));
                    }
                }
                points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
                parameters.push(t);
            }
            Ok(RenderBuffers::Curve { points, parameters })
        }
        Geometry::Brep(shape) => {
            let d = &shape.display;
            usage.add(
                Usage {
                    points: d.points.len() as u64,
                    ..Usage::array::<Point3>(d.points.len())?
                },
                l,
            )?;
            usage.add(Usage::array::<([u32; 3], u32)>(d.triangles.len())?, l)?;
            usage.add(Usage::array::<[u32; 2]>(d.segments.len())?, l)?;
            Ok(RenderBuffers::Mesh {
                vertices: d.points.clone(),
                triangles: d
                    .triangles
                    .iter()
                    .map(|[a, b, c, f]| ([*a, *b, *c], *f))
                    .collect(),
                edges: d.segments.iter().map(|[a, b, _]| [*a, *b]).collect(),
            })
        }
        Geometry::Mesh(m) => mesh_buffers(m, usage, l),
        Geometry::Hatch(h) => mesh_buffers(&h.display, usage, l),
        Geometry::Light(light) => cache(&Geometry::Point(light.display), usage, l),
        Geometry::NativeBrep(brep) => mesh_buffers(&brep.display, usage, l),
        Geometry::Surface(surface) => surface_buffers(surface, usage, l),
        _ => Err(Error::Unsupported("representation display adapter")),
    }
}
/// Tessellate a NURBS surface for display and picking only. The exact surface
/// remains the document geometry; this grid is never written back.
fn surface_buffers(
    surface: &om9_core::geometry::NurbsSurface,
    usage: &mut Usage,
    l: &Limits,
) -> Result<RenderBuffers> {
    const SAMPLES: usize = 12;
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let mut vertices = Vec::with_capacity(SAMPLES * SAMPLES);
    for i in 0..SAMPLES {
        let u = u0 + (u1 - u0) * i as f64 / (SAMPLES - 1) as f64;
        for j in 0..SAMPLES {
            let v = v0 + (v1 - v0) * j as f64 / (SAMPLES - 1) as f64;
            vertices.push(om9_core::surface_kernel::evaluate(surface, u, v, l)?);
        }
    }
    let mut triangles = Vec::with_capacity(2 * (SAMPLES - 1) * (SAMPLES - 1));
    for i in 0..SAMPLES - 1 {
        for j in 0..SAMPLES - 1 {
            let a = (i * SAMPLES + j) as u32;
            let b = ((i + 1) * SAMPLES + j) as u32;
            let c = ((i + 1) * SAMPLES + j + 1) as u32;
            let d = (i * SAMPLES + j + 1) as u32;
            triangles.push(([a, b, c], 0));
            triangles.push(([a, c, d], 0));
        }
    }
    let mut edges = Vec::new();
    for i in 0..SAMPLES - 1 {
        edges.push([(i * SAMPLES) as u32, ((i + 1) * SAMPLES) as u32]);
        edges.push([
            (i * SAMPLES + SAMPLES - 1) as u32,
            ((i + 1) * SAMPLES + SAMPLES - 1) as u32,
        ]);
    }
    for j in 0..SAMPLES - 1 {
        edges.push([j as u32, (j + 1) as u32]);
        edges.push([
            ((SAMPLES - 1) * SAMPLES + j) as u32,
            ((SAMPLES - 1) * SAMPLES + j + 1) as u32,
        ]);
    }
    usage.add(
        Usage {
            points: vertices.len() as u64,
            ..Usage::array::<Point3>(vertices.len())?
        },
        l,
    )?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles.len())?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges.len())?, l)?;
    Ok(RenderBuffers::Mesh {
        vertices,
        triangles,
        edges,
    })
}
fn mesh_buffers(
    m: &om9_core::geometry::Mesh,
    usage: &mut Usage,
    l: &Limits,
) -> Result<RenderBuffers> {
    let triangles = m.faces.iter().try_fold(0usize, |n, f| {
        n.checked_add(f.len() - 2)
            .ok_or(Error::Overflow("mesh triangles"))
    })?;
    let edges = m.faces.iter().try_fold(0usize, |n, f| {
        n.checked_add(f.len()).ok_or(Error::Overflow("mesh edges"))
    })?;
    usage.add(
        Usage {
            points: m.vertices.len() as u64,
            ..Usage::array::<Point3>(m.vertices.len())?
        },
        l,
    )?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges)?, l)?;
    let mut tris = Vec::with_capacity(triangles);
    let mut wire = Vec::with_capacity(edges);
    for (face, indices) in m.faces.iter().enumerate() {
        let face = u32::try_from(face).map_err(|_| Error::Overflow("native mesh face index"))?;
        for i in 1..indices.len() - 1 {
            tris.push(([indices[0], indices[i], indices[i + 1]], face));
        }
        for i in 0..indices.len() {
            wire.push([indices[i], indices[(i + 1) % indices.len()]]);
        }
    }
    Ok(RenderBuffers::Mesh {
        vertices: m.vertices.clone(),
        triangles: tris,
        edges: wire,
    })
}

struct Capture<'a> {
    document: &'a Arc<Document>,
    epoch: u64,
    limits: &'a Limits,
    usage: Usage,
    objects: Vec<SceneObject>,
    diagnostics: Vec<String>,
    shared: BTreeMap<(ObjectId, u64, u64), Arc<RenderBuffers>>,
    buffers_usage: Usage,
}
impl Capture<'_> {
    #[allow(clippy::too_many_arguments)] // All inherited occurrence state travels together.
    fn entity(
        &mut self,
        root: ObjectId,
        path: Vec<ObjectId>,
        entity: &Entity,
        transform: Transform,
        permissions: Permissions,
        reference_unlocked: bool,
        depth: usize,
    ) -> Result<()> {
        if depth > self.limits.max_depth {
            return Err(Error::Budget("scene instance depth"));
        }
        self.usage.add(
            Usage {
                nodes: 1,
                ..Usage::default()
            },
            self.limits,
        )?;
        let local = self.document.permissions_for_entity(entity, true)?;
        let permissions = combined(permissions, local);
        let reference_unlocked = reference_unlocked
            && self
                .document
                .permissions_for_entity(entity, false)?
                .reference_snappable;
        let transform = transform.compose(entity.placement)?;
        if let Geometry::Instance(id) = entity.geometry {
            let definition = self
                .document
                .definitions
                .get(&id)
                .ok_or(Error::Missing("scene definition"))?;
            for member in &definition.members {
                let mut p = path.clone();
                p.push(member.id);
                self.entity(
                    root,
                    p,
                    member,
                    transform,
                    permissions,
                    reference_unlocked,
                    depth + 1,
                )?;
            }
            return Ok(());
        }
        self.usage
            .add(Usage::array::<SceneObject>(1)?, self.limits)?;
        let key = (entity.id, entity.generation, entity.geometry_revision);
        let buffers = if let Some(buffers) = self.shared.get(&key) {
            buffers.clone()
        } else {
            match cache(&entity.geometry, &mut self.usage, self.limits) {
                Ok(buffers) => {
                    self.buffers_usage
                        .add(buffer_usage(&buffers)?, self.limits)?;
                    let buffers = Arc::new(buffers);
                    self.shared.insert(key, buffers.clone());
                    buffers
                }
                Err(Error::Unsupported(reason)) => {
                    self.diagnostics.push(format!("{}: {reason}", entity.id));
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        };
        let color = entity
            .attributes
            .color
            .or_else(|| {
                entity
                    .attributes
                    .layer
                    .and_then(|id| self.document.layers.get(&id).map(|l| l.color))
            })
            .unwrap_or([220, 220, 220, 255]);
        self.objects.push(SceneObject {
            root,
            path,
            leaf: entity.id,
            source: self.document.stamp(root)?,
            transform,
            preview: Transform::identity(),
            color,
            buffers,
            visible: permissions.visible,
            selectable: permissions.selectable,
            editable: permissions.editable,
            reference_snappable: permissions.reference_snappable,
            reference_unlocked,
        });
        Ok(())
    }
}
impl Scene {
    /// Owned render preparation snapshot: immutable document/buffers are shared; mutable picking
    /// caches are not. No UI/session/native owner is sent to the preparation worker.
    pub fn render_snapshot(&self) -> Self {
        Self {
            document: self.document.clone(),
            epoch: self.epoch,
            objects: self.objects.clone(),
            diagnostics: self.diagnostics.clone(),
            limits: self.limits,
            buffers_usage: self.buffers_usage,
            spatial: OnceLock::new(),
            spheres: OnceLock::new(),
        }
    }
    /// Picking index for this scene epoch; built once from the occurrence spheres.
    pub fn spatial_index(&self) -> &crate::spatial::SpatialIndex {
        let spheres = self.object_spheres();
        self.spatial.get_or_init(|| {
            let centres: Vec<[f64; 3]> = spheres.iter().map(|(centre, _)| *centre).collect();
            let radii: Vec<f64> = spheres.iter().map(|(_, radius)| *radius).collect();
            crate::spatial::SpatialIndex::build(&centres, &radii)
        })
    }

    /// World-space sphere per occurrence, aligned with `objects`.
    pub fn object_spheres(&self) -> &[([f64; 3], f64)] {
        self.spheres.get_or_init(|| {
            self.objects
                .iter()
                .map(|object| object.sphere_bounds().unwrap_or(([0.0; 3], 0.0)))
                .collect()
        })
    }

    /// Heap admission for the cache only; its document Arc is already owned by
    /// the coordinator/current or prepared Undo state. Buffers are counted once.
    pub fn cache_usage(&self) -> Result<Usage> {
        let mut usage = self.metadata_usage(true, 0)?;
        usage.add(self.buffers_usage, &self.limits)?;
        Ok(usage)
    }
    fn metadata_usage(&self, capacity: bool, extra: usize) -> Result<Usage> {
        let count = if capacity {
            self.objects.capacity()
        } else {
            self.objects.len()
        };
        let count = count
            .checked_add(extra)
            .ok_or(Error::Overflow("scene metadata count"))?;
        let mut usage = Usage {
            nodes: self.objects.len() as u64 + extra as u64,
            ..Usage::array::<SceneObject>(count)?
        };
        usage.add(Usage::array::<Self>(1)?, &self.limits)?;
        usage.add(
            Usage::array::<String>(if capacity {
                self.diagnostics.capacity()
            } else {
                self.diagnostics.len()
            })?,
            &self.limits,
        )?;
        for object in &self.objects {
            usage.add(
                Usage::array::<ObjectId>(if capacity {
                    object.path.capacity()
                } else {
                    object.path.len()
                })?,
                &self.limits,
            )?;
        }
        for message in &self.diagnostics {
            usage.add(
                Usage {
                    bytes: if capacity {
                        message.capacity()
                    } else {
                        message.len()
                    } as u64,
                    ..Usage::default()
                },
                &self.limits,
            )?;
        }
        Ok(usage)
    }
    pub fn update_usage(&self, addition: Option<&Entity>) -> Result<Usage> {
        let mut usage = self.metadata_usage(false, usize::from(addition.is_some()))?;
        let members = self
            .document
            .definitions
            .values()
            .try_fold(0usize, |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("scene member index"))
            })?;
        if members > 0 {
            usage.add(
                Usage::btree::<(DefinitionId, ObjectId), &Entity>(members)?,
                &self.limits,
            )?;
        }
        if let Some(entity) = addition {
            let count = match &entity.geometry {
                Geometry::Point(_) => 1,
                Geometry::Curve(Curve::Line { .. }) => 2,
                Geometry::Curve(Curve::Circle { .. }) => 720,
                Geometry::Curve(Curve::Polycurve { segments, .. }) => segments.len() + 1,
                Geometry::Curve(Curve::Nurbs(_)) => 257,
                Geometry::Surface(surface) => {
                    let mut local = Usage::default();
                    surface_buffers(surface, &mut local, &self.limits)?;
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(local, &self.limits)?;
                    return Ok(usage);
                }
                Geometry::Mesh(mesh) => {
                    let mut local = Usage::default();
                    mesh_buffers(mesh, &mut local, &self.limits)?;
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(local, &self.limits)?;
                    return Ok(usage);
                }
                Geometry::Brep(shape) => {
                    let display = &shape.display;
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(
                        Usage {
                            points: display.points.len() as u64,
                            ..Usage::array::<Point3>(display.points.len())?
                        },
                        &self.limits,
                    )?;
                    usage.add(
                        Usage::array::<([u32; 3], u32)>(display.triangles.len())?,
                        &self.limits,
                    )?;
                    usage.add(
                        Usage::array::<[u32; 2]>(display.segments.len())?,
                        &self.limits,
                    )?;
                    return Ok(usage);
                }
                _ => return Err(Error::Unsupported("plain root cache addition")),
            };
            usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
            usage.add(
                Usage {
                    bytes: 16,
                    points: count as u64,
                    ..Usage::default()
                },
                &self.limits,
            )?;
            usage.add(Usage::array::<Point3>(count)?, &self.limits)?;
            usage.add(
                if count == 1 {
                    Usage::array::<u32>(1)?
                } else {
                    Usage::array::<f64>(2)?
                },
                &self.limits,
            )?;
        }
        Ok(usage)
    }
    /// Reuse immutable geometry buffers only after proving all prior geometry,
    /// placements/identity/context unchanged. Optional addition is one Point/Line.
    pub fn update_owned(
        &self,
        document: Arc<Document>,
        epoch: u64,
        addition: Option<ObjectId>,
        limits: &Limits,
    ) -> Result<Self> {
        let old = &self.document;
        if old.id != document.id
            || old.owner_session != document.owner_session
            || old.epoch != document.epoch
            || old.session_generation != document.session_generation
            || old.context_revision != document.context_revision
            || old.page_unit != document.page_unit
            || old.definitions.len() != document.definitions.len()
            || old
                .entities
                .len()
                .checked_add(usize::from(addition.is_some()))
                != Some(document.entities.len())
        {
            return Err(Error::Stale("scene metadata owner/context"));
        }
        let same = |a: &Entity, b: &Entity| {
            a.id == b.id
                && a.generation == b.generation
                && a.geometry_revision == b.geometry_revision
                && a.geometry == b.geometry
                && a.placement == b.placement
        };
        for (id, e) in &old.entities {
            if !document.entities.get(id).is_some_and(|new| same(e, new)) {
                return Err(Error::Stale("scene metadata geometry"));
            }
        }
        for (id, definition) in &old.definitions {
            let new = document
                .definitions
                .get(id)
                .ok_or(Error::Stale("scene metadata definition"))?;
            if definition.members.len() != new.members.len()
                || !definition
                    .members
                    .iter()
                    .zip(&new.members)
                    .all(|(a, b)| same(a, b))
            {
                return Err(Error::Stale("scene metadata members"));
            }
        }
        let added = addition
            .map(|id| {
                document
                    .entities
                    .get(&id)
                    .filter(|_| !old.entities.contains_key(&id))
                    .ok_or(Error::Invalid("scene added root"))
            })
            .transpose()?;
        self.update_usage(added)?.check(limits)?;
        let mut members = BTreeMap::new();
        for (id, definition) in &document.definitions {
            for member in &definition.members {
                members.insert((*id, member.id), member);
            }
        }
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(self.objects.len() + usize::from(added.is_some()))
            .map_err(|_| Error::Budget("scene metadata allocation"))?;
        for object in &self.objects {
            let mut path = Vec::new();
            path.try_reserve_exact(object.path.len())
                .map_err(|_| Error::Budget("scene path allocation"))?;
            path.extend_from_slice(&object.path);
            let mut value = SceneObject {
                root: object.root,
                path,
                leaf: object.leaf,
                source: object.source,
                transform: object.transform,
                preview: object.preview,
                color: object.color,
                buffers: object.buffers.clone(),
                visible: object.visible,
                selectable: object.selectable,
                editable: object.editable,
                reference_snappable: object.reference_snappable,
                reference_unlocked: object.reference_unlocked,
            };
            let (leaf, permissions, unlocked) =
                metadata_leaf(&document, &members, object.root, &object.path)?;
            if leaf.id != object.leaf {
                return Err(Error::Stale("scene metadata leaf"));
            }
            value.source = document.stamp(object.root)?;
            value.color = leaf
                .attributes
                .color
                .or_else(|| {
                    leaf.attributes
                        .layer
                        .and_then(|id| document.layers.get(&id).map(|l| l.color))
                })
                .unwrap_or([220, 220, 220, 255]);
            value.visible = permissions.visible;
            value.selectable = permissions.selectable;
            value.editable = permissions.editable;
            value.reference_snappable = permissions.reference_snappable;
            value.reference_unlocked = unlocked;
            objects.push(value);
        }
        let mut buffers_usage = self.buffers_usage;
        if let Some(entity) = added {
            let mut usage = Usage::default();
            let buffers = cache(&entity.geometry, &mut usage, limits)?;
            buffers_usage.add(buffer_usage(&buffers)?, limits)?;
            let p = document.permissions(entity.id, true)?;
            let color = entity
                .attributes
                .color
                .or_else(|| {
                    entity
                        .attributes
                        .layer
                        .and_then(|id| document.layers.get(&id).map(|l| l.color))
                })
                .unwrap_or([220, 220, 220, 255]);
            let position = objects.partition_point(|object| object.root < entity.id);
            objects.insert(
                position,
                SceneObject {
                    root: entity.id,
                    path: vec![],
                    leaf: entity.id,
                    source: document.stamp(entity.id)?,
                    transform: entity.placement,
                    preview: Transform::identity(),
                    color,
                    buffers: Arc::new(buffers),
                    visible: p.visible,
                    selectable: p.selectable,
                    editable: p.editable,
                    reference_snappable: p.reference_snappable,
                    reference_unlocked: document.permissions(entity.id, false)?.reference_snappable,
                },
            );
        }
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(self.diagnostics.len())
            .map_err(|_| Error::Budget("scene diagnostics allocation"))?;
        for message in &self.diagnostics {
            let mut copy = String::new();
            copy.try_reserve_exact(message.len())
                .map_err(|_| Error::Budget("scene diagnostic allocation"))?;
            copy.push_str(message);
            diagnostics.push(copy);
        }
        Ok(Self {
            document,
            epoch,
            objects,
            diagnostics,
            limits: *limits,
            buffers_usage,
            spatial: OnceLock::new(),
            spheres: OnceLock::new(),
        })
    }
    pub fn capture(document: Arc<Document>, epoch: u64, limits: &Limits) -> Result<Self> {
        let usage = document.validate(limits)?;
        let plan = admission::admit(&document, limits, usage)?;
        Self::capture_admitted(document, epoch, limits, usage, plan)
    }
    pub fn capture_in_world(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        held: Usage,
    ) -> Result<Self> {
        let retained = coordinator.retained_usage_with_held_usage(held)?;
        let document = coordinator
            .world()
            .documents
            .get(&id)
            .ok_or(Error::Missing("scene world document"))?
            .clone();
        let plan = admission::admit(&document, coordinator.limits(), retained)?;
        let usage = document.validate(coordinator.limits())?;
        Self::capture_admitted(
            document,
            coordinator.epoch(),
            coordinator.limits(),
            usage,
            plan,
        )
    }
    fn capture_admitted(
        document: Arc<Document>,
        epoch: u64,
        limits: &Limits,
        usage: Usage,
        plan: admission::AllocationPlan,
    ) -> Result<Self> {
        let all = Permissions {
            visible: true,
            selectable: true,
            editable: true,
            reference_snappable: true,
        };
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(plan.objects)
            .map_err(|_| Error::Budget("scene object allocation"))?;
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(plan.diagnostics)
            .map_err(|_| Error::Budget("scene diagnostics allocation"))?;
        let mut capture = Capture {
            document: &document,
            epoch,
            limits,
            usage,
            objects,
            diagnostics,
            shared: BTreeMap::new(),
            buffers_usage: Usage::default(),
        };
        for entity in document.entities.values() {
            capture.entity(
                entity.id,
                vec![],
                entity,
                Transform::identity(),
                all,
                true,
                0,
            )?;
        }
        Ok(Self {
            document: document.clone(),
            epoch: capture.epoch,
            objects: capture.objects,
            diagnostics: capture.diagnostics,
            limits: *limits,
            buffers_usage: capture.buffers_usage,
            spatial: OnceLock::new(),
            spheres: OnceLock::new(),
        })
    }
    pub fn pick(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
    ) -> Result<Option<PickHit>> {
        crate::picking::pick(self, screen, camera, viewport, tolerance_pixels, intent)
    }

    /// Picks one kind of subobject: the object, a vertex, an edge or a face, nearest first.
    pub fn pick_kind(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
        kind: PickKind,
    ) -> Result<Option<PickHit>> {
        Ok(crate::picking::pick_candidates(
            self,
            screen,
            camera,
            viewport,
            tolerance_pixels,
            intent,
            kind,
            true,
        )?
        .into_iter()
        .next())
    }

    /// Every distinct object under the pointer, nearest first. The ambiguity popup lists these.
    pub fn pick_all(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
    ) -> Result<Vec<PickHit>> {
        crate::picking::pick_candidates(
            self,
            screen,
            camera,
            viewport,
            tolerance_pixels,
            intent,
            PickKind::Object,
            false,
        )
    }
    /// Read the exact root/member path represented by this scene occurrence.
    pub fn entity_for(&self, object: &SceneObject) -> Result<&Entity> {
        let mut entity = self
            .document
            .entities
            .get(&object.root)
            .ok_or(Error::Missing("scene root"))?;
        for id in &object.path {
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("scene member path"));
            };
            entity = self
                .document
                .definitions
                .get(&def)
                .ok_or(Error::Missing("scene definition"))?
                .members
                .iter()
                .find(|m| m.id == *id)
                .ok_or(Error::Missing("scene member"))?;
        }
        Ok(entity)
    }
}
