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
    sync::{Arc, OnceLock, Mutex},
};
mod admission;
mod picking_cache;
use picking_cache::{SceneMemory, SceneMemoryOwner};
pub use picking_cache::{ObjectSpheres, PickingIndex};

fn display_vector<T>(count: usize, reason: &'static str) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| Error::Budget(reason))?;
    Ok(values)
}

#[cfg(test)]
mod allocation_tests {
    #[test]
    fn snapshot_failure_releases_shared_owners_and_keeps_source_intact() {
        use om9_core::{Error, budget::Limits, geometry::Geometry, model::{Document, Entity}};
        use std::sync::Arc;
        let mut document = Document::new("snapshot failure");
        for index in 0..2 {
            let entity = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
            document.entities.insert(entity.id, entity);
        }
        let mut scene = super::Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
        scene.diagnostics.push("fixture warning".into());
        let document_owners = Arc::strong_count(&scene.document);
        let buffer_owners: Vec<_> = scene.objects.iter().map(|object| Arc::strong_count(&object.buffers)).collect();
        for fail_at in 0..5 {
            let mut step = 0;
            let result = scene.try_render_snapshot_checked(|| {
                let current = step;
                step += 1;
                if current == fail_at { Err(Error::Budget("injected snapshot failure")) } else { Ok(()) }
            });
            assert!(matches!(result, Err(Error::Budget("injected snapshot failure"))));
            assert_eq!(Arc::strong_count(&scene.document), document_owners);
            assert_eq!(scene.objects.iter().map(|object| Arc::strong_count(&object.buffers)).collect::<Vec<_>>(), buffer_owners);
            assert_eq!(scene.objects.len(), 2);
            assert_eq!(scene.diagnostics, ["fixture warning"]);
        }
        assert_eq!(scene.try_render_snapshot().unwrap().objects.len(), 2);
    }

    #[test]
    fn display_vector_reports_capacity_overflow() {
        assert!(matches!(super::display_vector::<u64>(usize::MAX, "display test"),
            Err(om9_core::Error::Budget("display test"))));
        assert_eq!(super::display_vector::<u64>(0, "display test").unwrap().len(), 0);
    }

    #[test]
    fn display_capacity_above_logical_lengths_is_charged_before_publication() {
        use om9_core::{Error, budget::{Limits, Usage}};
        let mut points = Vec::with_capacity(128);
        points.push([0., 0., 0.]);
        let buffer = super::RenderBuffers::Points { points, native_indices: vec![0] };
        let actual = super::buffer_usage(&buffer).unwrap();
        let estimated = Usage::array::<([f64; 3], u32)>(1).unwrap().bytes;
        assert!(actual.bytes > estimated);
        let mut usage = Usage { bytes: estimated, ..Usage::default() };
        let tight = Limits { max_bytes: actual.bytes - 1, ..Limits::default() };
        assert!(matches!(super::charge_buffer_capacity(&mut usage, 0, &buffer, &tight), Err(Error::Budget("bytes"))));
        assert_eq!(usage.bytes, estimated);
        let enough = Limits { max_bytes: actual.bytes, ..Limits::default() };
        assert_eq!(super::charge_buffer_capacity(&mut usage, 0, &buffer, &enough).unwrap().bytes, actual.bytes);
        assert_eq!(usage.bytes, actual.bytes);
    }
}

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
        self.sphere_bounds_from_local(self.local_bounds())
    }

    fn local_bounds(&self) -> Option<([f64; 3], [f64; 3])> {
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
            return None;
        }
        Some((min, max))
    }

    fn sphere_bounds_from_local(&self, bounds: Option<([f64; 3], [f64; 3])>) -> Result<([f64; 3], f64)> {
        let Some((min, max)) = bounds else { return Ok(([0.0; 3], 0.0)) };
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
    // Last: cache data must drop before its budget owner.
    memory: Arc<SceneMemoryOwner>,
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

/// `cache` admits logical lengths before allocation. Charge capacity beyond
/// those lengths (and the shared buffer owner) before publishing the scene.
fn charge_buffer_capacity(usage: &mut Usage, previous_bytes: u64, buffer: &RenderBuffers, limits: &Limits) -> Result<Usage> {
    let actual = buffer_usage(buffer)?;
    let estimated = usage.bytes.checked_sub(previous_bytes).ok_or(Error::Invalid("scene buffer accounting"))?;
    let extra = actual.bytes.saturating_sub(estimated);
    usage.add(Usage { bytes: extra, ..Usage::default() }, limits)?;
    Ok(actual)
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
// Walk leaves in display order without allocating a flattened copy.
fn walk_nested_segments(curve: &Curve, reverse: bool, visit: &mut impl FnMut(&Curve, bool) -> Result<()>) -> Result<()> {
    match curve {
        Curve::Polycurve { segments, reversed } => {
            let reverse = reverse ^ *reversed;
            if reverse {
                for segment in segments.iter().rev() { walk_nested_segments(segment, reverse, visit)?; }
            } else {
                for segment in segments { walk_nested_segments(segment, reverse, visit)?; }
            }
            Ok(())
        }
        _ => visit(curve, reverse),
    }
}

fn nested_display_count(curve: &Curve, limits: &Limits) -> Result<usize> {
    curve.validate(limits)?;
    let mut previous = None;
    let mut count = 1usize;
    walk_nested_segments(curve, false, &mut |segment, reverse| {
        validate_display_seam(segment, limits)?;
        let steps = polyline_display_steps(segment)?;
        let start = om9_core::curve_ops::point_at(segment, if reverse { 1. } else { 0. }, limits)?;
        let end = om9_core::curve_ops::point_at(segment, if reverse { 0. } else { 1. }, limits)?;
        let curved = !matches!(segment, Curve::Line { .. });
        if previous.is_some_and(|(point, previous_curved): (Point3, bool)| {
            if !curved && !previous_curved { point != start } else {
                (0..3).any(|axis| (point[axis]-start[axis]).abs() > 64. * f64::EPSILON * point[axis].abs().max(start[axis].abs()).max(1.))
            }
        }) {
            return Err(Error::Unsupported("nested polycurve display requires connected lines"));
        }
        previous = Some((end, curved));
        count = count.checked_add(steps).ok_or(Error::Overflow("nested polycurve vertices"))?;
        Ok(())
    })?;
    Ok(count)
}

fn has_nested_segments(curve: &Curve) -> bool {
    matches!(curve, Curve::Polycurve { segments, .. } if segments.iter().any(|segment| matches!(segment, Curve::Polycurve { .. })))
}

/// A displayed polycurve is a connected list of direct lines, arcs or continuous NURBS.
/// Keep this admission check shared by the allocation plan and display cache;
/// connecting arbitrary polycurve segments would invent geometry between gaps.
fn connected_polyline_segments<'a>(curve: &'a Curve, limits: &Limits) -> Result<&'a [Curve]> {
    let Curve::Polycurve {
        segments,
        ..
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
        let (start, end, arc) = match segment {
            Curve::Line { start, end, .. } => (*start, *end, false),
            Curve::Circle { domain, .. } |
            Curve::Nurbs(om9_core::geometry::NurbsCurve { domain, seam: None, .. }) => {
                polyline_display_steps(segment)?;
                let evaluator = CurveEvaluator::capture(segment, limits)?;
                let reversed = matches!(segment, Curve::Nurbs(n) if n.reversed);
                (
                    evaluator.evaluate(Parameter::Native(domain[usize::from(reversed)]), 0, Transform::identity())?[0],
                    evaluator.evaluate(Parameter::Native(domain[usize::from(!reversed)]), 0, Transform::identity())?[0],
                    true,
                )
            }
            Curve::Nurbs(n) if n.seam.is_some() => {
                polyline_display_steps(segment)?;
                validate_display_seam(segment, limits)?;
                let p = om9_core::curve_ops::point_at(segment, 0., limits)?;
                (p, p, true)
            }
            _ => {
                return Err(Error::Unsupported(
                    "polycurve display requires direct line, arc or continuous NURBS segments",
                ));
            }
        };
        if previous_end.is_some_and(|(previous, previous_arc): (Point3, bool)| {
            if !arc && !previous_arc {
                previous != start
            } else {
                (0..3).any(|axis| {
                    (previous[axis] - start[axis]).abs()
                        > 64.0 * f64::EPSILON * previous[axis].abs().max(start[axis].abs()).max(1.0)
                })
            }
        }) {
            return Err(Error::Unsupported(
                "polycurve display requires connected line segments",
            ));
        }
        previous_end = Some((end, arc));
    }
    Ok(segments)
}

fn walk_nurbs_spans(n: &om9_core::geometry::NurbsCurve, reverse: bool, visit: &mut impl FnMut(f64, f64) -> Result<()>) -> Result<()> {
    let [min, max] = n.domain;
    let seam = n.seam.map(|s| if s == max { min } else { s });
    let intervals = match (seam, reverse) {
        (None, _) => [(min, max), (min, min)],
        (Some(s), false) => [(s, max), (min, s)],
        (Some(s), true) => [(min, s), (s, max)],
    };
    for (from, to) in intervals {
        for j in 0..n.knots.len()-1 {
            let i = if reverse { n.knots.len()-2-j } else { j };
            let low = n.knots[i].max(from);
            let high = n.knots[i+1].min(to);
            if low < high { if reverse { visit(high, low)?; } else { visit(low, high)?; } }
        }
    }
    Ok(())
}

fn validate_display_seam(curve: &Curve, limits: &Limits) -> Result<()> {
    let Curve::Nurbs(n) = curve else { return Ok(()); };
    if n.seam.is_none() { return Ok(()); }
    let evaluator = CurveEvaluator::capture(curve, limits)?;
    let a = evaluator.evaluate(Parameter::Native(n.domain[0]), 0, Transform::identity())?[0];
    let b = evaluator.evaluate(Parameter::Native(n.domain[1]), 0, Transform::identity())?[0];
    if (0..3).any(|axis| (a[axis]-b[axis]).abs() > 64. * f64::EPSILON * a[axis].abs().max(b[axis].abs()).max(1.)) {
        return Err(Error::Unsupported("reseamed display requires closed NURBS"));
    }
    Ok(())
}

fn polyline_display_steps(curve: &Curve) -> Result<usize> {
    match curve {
        Curve::Line { .. } => Ok(1),
        Curve::Circle {
            radius,
            domain,
            angle_domain,
            ..
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            let angle = 2.0 * (1.0 - (0.005 / radius).min(1.0)).acos();
            let estimate = ((angles[1] - angles[0]) / angle).ceil().max(16.0);
            if !estimate.is_finite() || estimate > 65535.0 {
                return Err(Error::Budget("circle display chord samples"));
            }
            Ok(estimate as usize)
        }
        Curve::Nurbs(n) => {
            // A fully repeated internal knot can be discontinuous; a single
            // display strip would invent a connector across such a break.
            let mut i = 0;
            while i < n.knots.len() {
                let knot = n.knots[i];
                let mut end = i + 1;
                while end < n.knots.len() && n.knots[end] == knot { end += 1; }
                if knot > n.domain[0] && knot < n.domain[1] && end - i > n.degree {
                    return Err(Error::Unsupported("discontinuous polycurve NURBS segment"));
                }
                i = end;
            }
            let mut spans = 0usize;
            walk_nurbs_spans(n, false, &mut |_, _| { spans = spans.checked_add(1).ok_or(Error::Overflow("NURBS display spans"))?; Ok(()) })?;
            spans.checked_mul(256).ok_or(Error::Overflow("NURBS display samples"))
        }
        _ => Err(Error::Unsupported("polycurve display segment")),
    }
}

fn polyline_display_count(segments: &[Curve]) -> Result<usize> {
    segments.iter().try_fold(1usize, |count, segment| {
        count
            .checked_add(polyline_display_steps(segment)?)
            .ok_or(Error::Overflow("polyline display vertices"))
    })
}

fn append_display_segment(segment: &Curve, reverse: bool, points: &mut Vec<Point3>, limits: &Limits) -> Result<()> {
    if let Curve::Line { start, end, .. } = segment {
        let (start, end) = if reverse { (*end, *start) } else { (*start, *end) };
        if points.is_empty() { points.push(start); }
        points.push(end);
    } else if let Curve::Nurbs(n) = segment {
        let evaluator = CurveEvaluator::capture(segment, limits)?;
        let reverse = reverse ^ n.reversed;
        walk_nurbs_spans(n, reverse, &mut |start, end| {
            if points.is_empty() { points.push(evaluator.evaluate(Parameter::Native(start), 0, Transform::identity())?[0]); }
            for i in 1..=256 {
                let t = if i == 256 { end } else { start + (end-start)*i as f64/256. };
                points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
            }
            Ok(())
        })?;
    } else if let Curve::Circle { domain, .. } = segment {
        let evaluator = CurveEvaluator::capture(segment, limits)?;
        let steps = polyline_display_steps(segment)?;
        let (start, end) = if reverse { (domain[1], domain[0]) } else { (domain[0], domain[1]) };
        if points.is_empty() { points.push(evaluator.evaluate(Parameter::Native(start), 0, Transform::identity())?[0]); }
        for i in 1..=steps {
            let t = if i == steps { end } else { start + (end-start)*i as f64/steps as f64 };
            points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
        }
    } else {
        return Err(Error::Unsupported("polycurve display segment"));
    }
    Ok(())
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
            if has_nested_segments(curve) {
                let count = nested_display_count(curve, l)?;
                usage.add(Usage { points: count as u64, ..Usage::array::<Point3>(count)? }, l)?;
                let mut points = Vec::new();
                points.try_reserve_exact(count).map_err(|_| Error::Budget("nested polycurve display allocation"))?;
                walk_nested_segments(curve, false, &mut |segment, reverse| {
                    append_display_segment(segment, reverse, &mut points, l)
                })?;
                return Ok(RenderBuffers::Curve { points, parameters: Vec::new() });
            }
            // An annotation stores independent line strokes. Render each stroke
            // explicitly; joining gaps would introduce lines that are not in the model.
            if let Curve::Polycurve{segments,reversed:false}=curve
                && connected_polyline_segments(curve,l).is_err()&&segments.iter().all(|s|matches!(s,Curve::Line{..})) {
                let count=segments.len().checked_mul(2).ok_or(Error::Overflow("line stroke vertices"))?;
                usage.add(Usage{points:count as u64,..Usage::array::<Point3>(count)?},l)?;
                usage.add(Usage::array::<[u32;2]>(segments.len())?,l)?;
                let mut vertices=display_vector(count, "stroke display allocation")?;
                let mut edges=display_vector(segments.len(), "stroke edge allocation")?;
                for segment in segments {if let Curve::Line{start,end,..}=segment{let i=u32::try_from(vertices.len()).map_err(|_|Error::Overflow("stroke vertex index"))?;vertices.extend([*start,*end]);edges.push([i,i+1]);}}
                return Ok(RenderBuffers::Mesh{vertices,triangles:vec![],edges});
            }
            let segments = connected_polyline_segments(curve, l)?;
            let count = polyline_display_count(segments)?;
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
                append_display_segment(segment, false, &mut points, l)?;
            }
            if matches!(curve, Curve::Polycurve { reversed: true, .. }) { points.reverse(); }
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
                Curve::Nurbs(c) if c.periodic => {c.certify_repeated_periodic_basis(l)?;(c.domain,256)},
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
            let mut points = display_vector(count, "curve display allocation")?;
            let mut parameters = display_vector(count, "curve parameter allocation")?;
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
            let mut vertices = display_vector(d.points.len(), "BRep display vertices")?;
            let mut triangles = display_vector(d.triangles.len(), "BRep display triangles")?;
            let mut edges = display_vector(d.segments.len(), "BRep display edges")?;
            vertices.extend_from_slice(&d.points);
            triangles.extend(d.triangles.iter().map(|[a, b, c, f]| ([*a, *b, *c], *f)));
            edges.extend(d.segments.iter().map(|[a, b, _]| [*a, *b]));
            Ok(RenderBuffers::Mesh { vertices, triangles, edges })
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
const SURFACE_SAMPLES: usize = 12;
fn surface_buffer_usage(usage: &mut Usage, l: &Limits) -> Result<()> {
    let vertices = SURFACE_SAMPLES * SURFACE_SAMPLES;
    let triangles = 2 * (SURFACE_SAMPLES - 1) * (SURFACE_SAMPLES - 1);
    let edges = 4 * (SURFACE_SAMPLES - 1);
    usage.add(Usage { points: vertices as u64, ..Usage::array::<Point3>(vertices)? }, l)?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges)?, l)
}
fn surface_buffers(
    surface: &om9_core::geometry::NurbsSurface,
    usage: &mut Usage,
    l: &Limits,
) -> Result<RenderBuffers> {
    const SAMPLES: usize = SURFACE_SAMPLES;
    surface_buffer_usage(usage, l)?;
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let mut vertices = display_vector(SAMPLES * SAMPLES, "surface display vertices")?;
    for i in 0..SAMPLES {
        let u = u0 + (u1 - u0) * i as f64 / (SAMPLES - 1) as f64;
        for j in 0..SAMPLES {
            let v = v0 + (v1 - v0) * j as f64 / (SAMPLES - 1) as f64;
            vertices.push(om9_core::surface_kernel::evaluate(surface, u, v, l)?);
        }
    }
    let mut triangles = display_vector(2 * (SAMPLES - 1) * (SAMPLES - 1), "surface display triangles")?;
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
    let mut edges = display_vector(4 * (SAMPLES - 1), "surface display edges")?;
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
    Ok(RenderBuffers::Mesh {
        vertices,
        triangles,
        edges,
    })
}
fn mesh_buffer_usage(m: &om9_core::geometry::Mesh, usage: &mut Usage, l: &Limits) -> Result<(usize, usize)> {
    let triangles = m.faces.iter().try_fold(0usize, |n, f| {
        let count = f.len().checked_sub(2).ok_or(Error::Invalid("mesh face"))?;
        n.checked_add(count).ok_or(Error::Overflow("mesh triangles"))
    })?;
    let edges = m.faces.iter().try_fold(0usize, |n, f| {
        n.checked_add(f.len()).ok_or(Error::Overflow("mesh edges"))
    })?;
    usage.add(Usage { points: m.vertices.len() as u64, ..Usage::array::<Point3>(m.vertices.len())? }, l)?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges)?, l)?;
    Ok((triangles, edges))
}
fn mesh_buffers(
    m: &om9_core::geometry::Mesh,
    usage: &mut Usage,
    l: &Limits,
) -> Result<RenderBuffers> {
    let (triangles, edges) = mesh_buffer_usage(m, usage, l)?;
    let mut tris = display_vector(triangles, "mesh display triangles")?;
    let mut wire = display_vector(edges, "mesh display edges")?;
    let mut vertices = display_vector(m.vertices.len(), "mesh display vertices")?;
    vertices.extend_from_slice(&m.vertices);
    for (face, indices) in m.faces.iter().enumerate() {
        let face = u32::try_from(face).map_err(|_| Error::Overflow("native mesh face index"))?;
        if let [a, b, c, d] = indices.as_slice() {
            let points = [*a, *b, *c, *d].map(|index| m.vertices[index as usize]);
            if matches!(om9_core::geometry::quad_diagonal_ac(points), Ok(false)) {
                tris.push(([*a, *b, *d], face));
                tris.push(([*b, *c, *d], face));
            } else {
                tris.push(([*a, *b, *c], face));
                tris.push(([*a, *c, *d], face));
            }
        } else {
            for i in 1..indices.len() - 1 {
                tris.push(([indices[0], indices[i], indices[i + 1]], face));
            }
        }
        for i in 0..indices.len() {
            wire.push([indices[i], indices[(i + 1) % indices.len()]]);
        }
    }
    Ok(RenderBuffers::Mesh {
        vertices,
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
            let previous_bytes = self.usage.bytes;
            match cache(&entity.geometry, &mut self.usage, self.limits) {
                Ok(buffers) => {
                    let actual = charge_buffer_capacity(&mut self.usage, previous_bytes, &buffers, self.limits)?;
                    self.buffers_usage
                        .add(actual, self.limits)?;
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
        self.try_render_snapshot().expect("scene snapshot allocation")
    }
    /// Construct a worker snapshot without aborting when its metadata cannot
    /// be allocated. Geometry buffers remain shared with the source scene.
    pub fn try_render_snapshot(&self) -> Result<Self> {
        self.try_render_snapshot_checked(|| Ok(()))
    }
    fn try_render_snapshot_checked(&self, mut check: impl FnMut() -> Result<()>) -> Result<Self> {
        let mut objects = Vec::new();
        check()?;
        objects.try_reserve_exact(self.objects.len()).map_err(|_| Error::Budget("scene snapshot objects"))?;
        for object in &self.objects {
            let mut path = Vec::new();
            check()?;
            path.try_reserve_exact(object.path.len()).map_err(|_| Error::Budget("scene snapshot path"))?;
            path.extend_from_slice(&object.path);
            objects.push(SceneObject {
                root: object.root, path, leaf: object.leaf, source: object.source,
                transform: object.transform, preview: object.preview, color: object.color,
                buffers: object.buffers.clone(), visible: object.visible, selectable: object.selectable,
                editable: object.editable, reference_snappable: object.reference_snappable,
                reference_unlocked: object.reference_unlocked,
            });
        }
        let mut diagnostics = Vec::new();
        check()?;
        diagnostics.try_reserve_exact(self.diagnostics.len()).map_err(|_| Error::Budget("scene snapshot diagnostics"))?;
        for message in &self.diagnostics {
            let mut copy = String::new();
            check()?;
            copy.try_reserve_exact(message.len()).map_err(|_| Error::Budget("scene snapshot diagnostic"))?;
            copy.push_str(message);
            diagnostics.push(copy);
        }
        Ok(Self {
            document: self.document.clone(),
            epoch: self.epoch,
            objects,
            diagnostics,
            limits: self.limits,
            buffers_usage: self.buffers_usage,
            memory: Arc::new(SceneMemoryOwner::default()),
        })
    }

    /// Heap admission for the cache only; its document Arc is already owned by
    /// the coordinator/current or prepared Undo state. Buffers are counted once.
    pub fn cache_usage(&self) -> Result<Usage> {
        let mut usage = self.metadata_usage(true, 0)?;
        let (picking, _) = self.memory_usage(usage.bytes)?;
        usage.add(self.buffers_usage, &self.limits)?;
        usage.add(Usage { bytes: picking, ..Usage::default() }, &self.limits)?;
        Ok(usage)
    }
    /// Account cache storage alongside a separately validated document.
    pub fn cache_held_usage(&self) -> Result<Usage> {
        let mut usage = self.metadata_usage(true, 0)?;
        let (picking, registered) = self.memory_usage(usage.bytes)?;
        usage.add(self.buffers_usage, &self.limits)?;
        usage.add(Usage { bytes: picking, ..Usage::default() }, &self.limits)?;
        usage.bytes = usage.bytes.checked_sub(registered).ok_or(Error::Invalid("scene metadata accounting"))?;
        usage.points = 0;
        Ok(usage)
    }
    /// Bytes held by the distinct shared render buffers of this scene.
    /// Scene metadata and lazily built picking indices are not included.
    pub fn shared_buffer_bytes(&self) -> u64 {
        self.buffers_usage.bytes
    }
    /// Requested metadata for a fresh render snapshot. Shared geometry and the
    /// source scene's lazy picking caches are not copied. Allocator capacity
    /// above the request must be reconciled after construction.
    pub fn render_snapshot_metadata_estimate(&self) -> Result<Usage> {
        self.metadata_usage(false, 0)
    }
    /// Actual metadata capacity, excluding geometry and lazy picking caches.
    pub fn metadata_storage_usage(&self) -> Result<Usage> {
        self.metadata_usage(true, 0)
    }
    /// Estimate lazy bounds/index and one mesh's transformed-vertex workspace
    /// for a fresh nearest-pick snapshot. Excludes allocator overhead and
    /// implementation-dependent sorting, evaluation and query allocations.
    pub fn nearest_pick_scratch_estimate(&self) -> Result<Usage> {
        let mut usage = self.nearest_pick_query_scratch_estimate()?;
        let state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if state.reservation.is_none() {
            usage.add(Usage { bytes: self.picking_arena_bytes()?, ..Usage::default() },
                &Limits { max_bytes: u64::MAX, ..Limits::default() })?;
        }
        Ok(usage)
    }
    /// A fresh worker owns its own arena; the source's cache is not copied.
    pub fn render_snapshot_pick_scratch_estimate(&self) -> Result<Usage> {
        let mut usage = self.nearest_pick_query_scratch_estimate()?;
        usage.add(Usage { bytes: self.picking_arena_bytes()?, ..Usage::default() },
            &Limits { max_bytes: u64::MAX, ..Limits::default() })?;
        Ok(usage)
    }
    pub fn nearest_pick_query_scratch_estimate(&self) -> Result<Usage> {
        let limits = Limits { max_bytes: u64::MAX, ..Limits::default() };
        let mut usage = Usage::array::<usize>(self.objects.len())?;
        usage.add(Usage::array::<u32>(self.objects.len())?, &limits)?;
        let max_vertices = self.objects.iter().filter_map(|object| match object.buffers.as_ref() {
            RenderBuffers::Mesh { vertices, .. } => Some(vertices.len()),
            _ => None,
        }).max().unwrap_or(0);
        usage.add(Usage::array::<Option<Point3>>(max_vertices)?, &limits)?;
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
        usage.add(Usage::array::<Mutex<SceneMemory>>(1)?, &self.limits)?;
        usage.add(Usage::array::<usize>(2)?, &self.limits)?;
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
                Geometry::Curve(curve @ Curve::Polycurve { segments, .. }) => {
                    match connected_polyline_segments(curve, &self.limits) {
                        Ok(connected) => polyline_display_count(connected)?,
                        Err(_) if segments.iter().all(|s| matches!(s, Curve::Line { .. })) => {
                            usage.add(Usage::array::<[u32; 2]>(segments.len())?, &self.limits)?;
                            segments.len().checked_mul(2).ok_or(Error::Overflow("stroke vertices"))?
                        }
                        Err(_) => segments.len().checked_add(1).ok_or(Error::Overflow("polyline display vertices"))?,
                    }
                },
                Geometry::Curve(Curve::Nurbs(_)) => 257,
                Geometry::Surface(_) => {
                    let mut local = Usage::default();
                    surface_buffer_usage(&mut local, &self.limits)?;
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(local, &self.limits)?;
                    return Ok(usage);
                }
                Geometry::Mesh(mesh) => {
                    let mut local = Usage::default();
                    mesh_buffer_usage(mesh, &mut local, &self.limits)?;
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
    /// Extra scene ownership held beside a document transaction. Source and
    /// scene each obey the point limit independently; all bytes remain charged.
    pub fn update_held_usage(&self, addition: Option<&Entity>) -> Result<Usage> {
        let mut scene_points = self.buffers_usage.points;
        self.update_held_usage_counted(addition, &mut scene_points)
    }
    /// Carry the scene point count across a batch before any transaction is
    /// published. The caller starts with `cache_usage()?.points`.
    pub fn update_held_usage_counted(&self, addition: Option<&Entity>, scene_points: &mut u64) -> Result<Usage> {
        if *scene_points < self.buffers_usage.points {
            return Err(Error::Invalid("scene point baseline"));
        }
        let mut usage = self.update_usage(addition)?;
        let mut total = Usage { points: *scene_points, ..Usage::default() };
        total.add(Usage { points: usage.points, ..Usage::default() }, &self.limits)?;
        *scene_points = total.points;
        usage.points = 0;
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
            let actual = charge_buffer_capacity(&mut usage, 0, &buffers, limits)?;
            buffers_usage.add(actual, limits)?;
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
            memory: Arc::new(SceneMemoryOwner::default()),
        })
    }
    /// Incremental scene plus its newly retained bytes while `self` remains
    /// alive. `update_owned` shares all existing render buffers; metadata and
    /// any added buffer are independent. Planner scratch is not retained here.
    pub fn update_owned_with_retained_bytes(
        &self, document: Arc<Document>, epoch: u64, addition: Option<ObjectId>, limits: &Limits,
    ) -> Result<(Self, u64)> {
        let next = self.update_owned(document, epoch, addition, limits)?;
        let bytes = next.cache_usage()?.bytes.checked_sub(self.buffers_usage.bytes)
            .ok_or(Error::Invalid("incremental scene shared buffer accounting"))?;
        Ok((next, bytes))
    }

    pub fn capture(document: Arc<Document>, epoch: u64, limits: &Limits) -> Result<Self> {
        let usage = document.validate(limits)?;
        let plan = admission::admit(&document, limits, usage)?;
        Self::capture_admitted(document, epoch, limits, usage, plan)
    }
    /// Scratch for the scene planner only, excluding document validation and
    /// the document itself. Use `candidate_preflight_scratch` before asking
    /// for a candidate scene peak.
    pub fn candidate_planning_scratch(document: &Document) -> Result<Usage> {
        admission::planning_scratch(document)
    }
    /// Validation and planning run sequentially in `candidate_scene_peak`,
    /// so their maximum is held alongside the already-owned document.
    pub fn candidate_preflight_scratch(document: &Document, limits: &Limits) -> Result<Usage> {
        let planning = admission::planning_scratch(document)?;
        let validation = document.validation_scratch_usage(limits)?;
        Ok(Usage { bytes: planning.bytes.max(validation.bytes), ..Usage::default() })
    }
    /// Estimate scene construction while the caller holds preflight scratch
    /// and document reservations. This is an admission estimate, not measured RSS.
    pub fn candidate_scene_peak(document: &Document, limits: &Limits) -> Result<u64> {
        let owned = document.validate(limits)?;
        Ok(admission::admit(document, limits, owned)?.peak_bytes)
    }
    pub fn capture_in_world(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        held: Usage,
    ) -> Result<Self> {
        Self::capture_in_world_with_physical_samples(coordinator, id, held, || None)
    }
    /// Sample immediately before planner, validation and construction admission. A `None`
    /// sample retains managed-only admission for platforms without a provider.
    pub fn capture_in_world_with_physical_samples(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        held: Usage,
        sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<Self> {
        let document = coordinator
            .world()
            .documents
            .get(&id)
            .ok_or(Error::Missing("scene world document"))?
            .clone();
        Self::capture_retained_with_physical_samples(coordinator, document, coordinator.epoch(), held, sample)
    }
    /// Capture a version already retained by the coordinator, including a
    /// staged Undo/Redo candidate. Its caller accounts for other held state.
    pub fn capture_retained_with_physical_samples(
        coordinator: &TransactionCoordinator,
        document: Arc<Document>,
        epoch: u64,
        held: Usage,
        sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<Self> {
        let (scene, _reservation) = Self::capture_retained_reserved_with_physical_samples(coordinator, document, epoch, held, sample)?;
        Ok(scene)
    }

    /// Return the construction reservation with the finished scene so its
    /// caller can keep the account charged until publication or handoff.
    pub fn capture_retained_reserved_with_physical_samples(
        coordinator: &TransactionCoordinator,
        document: Arc<Document>,
        epoch: u64,
        held: Usage,
        mut sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<(Self, om9_core::transaction::ExternalMemoryReservation)> {
        if !coordinator.owns_retained_document(&document) {
            return Err(Error::Invalid("scene document is not retained"));
        }
        let (retained, limits) = coordinator.retained_usage_and_limits_with_held(held)?;
        let plan = {
            // Reserve the planner's distinct-source seen set before it grows.
            // Its lifetime ends when `admit` returns, before capture starts.
            let scratch = admission::planning_scratch(&document)?;
            let _planning = coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes, sample(), held,
            )?;
            admission::admit(&document, &limits, retained)?
        };
        let usage = {
            // The document itself is already charged by the coordinator; only
            // reserve the extra temporary validation storage here.
            let scratch = document.validation_scratch_usage(&limits)?;
            let _validation = coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes.max(1), sample(), held,
            )?;
            document.validate(&limits)?
        };
        // Admission rechecks current shared reservations, including caller-held
        // allocations, and keeps the construction peak reserved until capture
        // completes and is handed to the caller alongside the scene.
        let mut reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, plan.peak_bytes, sample(), held,
        )?;
        let scene = Self::capture_admitted(
            document,
            epoch,
            &limits,
            usage,
            plan,
        )?;
        let actual_bytes = scene.cache_held_usage()?.bytes;
        if actual_bytes > reservation.bytes() {
            let additional = actual_bytes - reservation.bytes();
            coordinator.grow_external_memory_with_held(&mut reservation, additional, sample(), held)?;
        }
        Ok((scene, reservation))
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
        // The source document was independently validated above. Display
        // points use the same per-scene cap without charging source points a
        // second time; memory bytes still cover both live owners.
        let mut display_usage = usage;
        display_usage.points = 0;
        let mut capture = Capture {
            document: &document,
            epoch,
            limits,
            usage: display_usage,
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
            memory: Arc::new(SceneMemoryOwner::default()),
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

    /// Nearest matching subobject for each distinct root object under the pointer.
    pub fn pick_kind_all(
        &self, screen: [f64; 2], camera: &Camera, viewport: Viewport,
        tolerance_pixels: f64, intent: PickIntent, kind: PickKind,
    ) -> Result<Vec<PickHit>> {
        crate::picking::pick_candidates(self, screen, camera, viewport, tolerance_pixels, intent, kind, false)
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
