// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino-like object semantics (`OM9-L2-02-01`).
//!
//! This module owns the *semantic* layer over the portable geometry records: which object kind a
//! curve/surface actually is, whether a curve is closed or planar under a stated tolerance, whether
//! a surface domain/seam/periodicity is internally consistent, what a trim or extrusion record means,
//! which edges are pickable topology rather than display-only isocurves, whether a B-Rep body is a
//! closed solid or an open shell, and whether a block instance shares a definition without sharing a
//! transform.
//!
//! It never re-implements OCCT evaluation. It composes the owned geometry (`geometry`,
//! `curve_build`, `surface_kernel`, `brep_kernel`) so a UI or exchange adapter has one typed answer
//! instead of guessing from a mesh or a display cache.

use crate::{
    Error, Result,
    brep_kernel::{BrepBody, ShapeKind},
    budget::Limits,
    curve_build,
    evaluation::{Parameter, evaluate_curve},
    geometry::{Curve, Geometry, NurbsCurve, NurbsSurface, Point3, Transform},
    identity::{DefinitionId, ObjectId},
    model::{Document, EXTRUSION_EXTENSION_SLOT, Entity, Extension, TRIM_SURFACE_EXTENSION_SLOT},
    surface_kernel::{self, SurfaceIso},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// The semantic object kind a document entity presents. A mesh is never reported as a NURBS or
/// B-Rep kind, and a trimmed surface is never reported as its untrimmed underlying surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticKind {
    Point,
    Line,
    Circle,
    Polyline,
    Polycurve,
    NurbsCurve,
    NurbsSurface,
    TrimmedSurface,
    Extrusion,
    Mesh,
    PointCloud,
    Brep,
    Instance,
}

/// Classify a geometry record without consulting entity metadata (extensions).
pub fn geometry_semantic_kind(geometry: &Geometry) -> SemanticKind {
    match geometry {
        Geometry::Point(_) => SemanticKind::Point,
        Geometry::Curve(curve) => curve_semantic_kind(curve),
        Geometry::Surface(_) => SemanticKind::NurbsSurface,
        Geometry::Brep(_) | Geometry::NativeBrep(_) => SemanticKind::Brep,
        Geometry::Mesh(_) => SemanticKind::Mesh,
        Geometry::Cloud(_) => SemanticKind::PointCloud,
        Geometry::Instance(_) => SemanticKind::Instance,
    }
}

/// Classify a curve by its own structure. A polycurve of connected line segments is a polyline;
/// any other polycurve remains a polycurve.
pub fn curve_semantic_kind(curve: &Curve) -> SemanticKind {
    match curve {
        Curve::Line { .. } => SemanticKind::Line,
        Curve::Circle { .. } => SemanticKind::Circle,
        Curve::Nurbs(_) => SemanticKind::NurbsCurve,
        Curve::Polycurve { .. } => {
            if is_line_polyline(curve, 1.0e-9) {
                SemanticKind::Polyline
            } else {
                SemanticKind::Polycurve
            }
        }
    }
}

/// Classify an entity: a persistent trim or extrusion record overrides the base geometry kind.
pub fn entity_semantic_kind(entity: &Entity) -> Result<SemanticKind> {
    if entity.extensions.contains_key(TRIM_SURFACE_EXTENSION_SLOT) {
        let trimmed = trimmed_surface_of(entity, &Limits::default())?
            .ok_or(Error::Invalid("trimmed surface payload"))?;
        match &entity.geometry {
            Geometry::Surface(surface) if surface == &trimmed.underlying => {
                Ok(SemanticKind::TrimmedSurface)
            }
            Geometry::Surface(_) => Err(Error::Invalid("trimmed surface underlying mismatch")),
            _ => Err(Error::Invalid("trimmed surface requires surface geometry")),
        }
    } else if entity.extensions.contains_key(EXTRUSION_EXTENSION_SLOT) {
        let extrusion =
            extrusion_of(entity, &Limits::default())?.ok_or(Error::Invalid("extrusion payload"))?;
        match &entity.geometry {
            Geometry::Curve(curve) if curve == &extrusion.profile => Ok(SemanticKind::Extrusion),
            Geometry::Curve(_) => Err(Error::Invalid("extrusion profile mismatch")),
            _ => Err(Error::Invalid("extrusion requires profile geometry")),
        }
    } else {
        Ok(geometry_semantic_kind(&entity.geometry))
    }
}

/* -------------------------------- curves -------------------------------- */

/// The structural form of a curve. `Point` is a `Geometry::Point`, not a curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveForm {
    Line,
    Circle,
    Polyline,
    Polycurve,
    Nurbs,
}

/// Curve semantics measured under one explicit tolerance policy.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveSemantics {
    pub form: CurveForm,
    /// `distance(start, end) <= tolerance`; never raw equality.
    pub closed: bool,
    pub planar: bool,
    pub endpoints: [Point3; 2],
    pub tolerance: f64,
}

fn check_tolerance(tolerance: f64) -> Result<()> {
    if !tolerance.is_finite() || tolerance <= 0.0 || tolerance > 1.0e3 {
        return Err(Error::Invalid("semantic tolerance"));
    }
    Ok(())
}

fn distance(a: Point3, b: Point3) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}

fn is_line_polyline(curve: &Curve, tolerance: f64) -> bool {
    let Curve::Polycurve { segments, reversed } = curve else {
        return false;
    };
    if segments.is_empty() {
        return false;
    }
    let mut expected: Option<Point3> = None;
    let mut visit = |segment: &Curve| -> bool {
        let Curve::Line { start, end, .. } = segment else {
            return false;
        };
        if let Some(previous) = expected
            && distance(previous, *start) > tolerance
        {
            return false;
        }
        expected = Some(*end);
        true
    };
    if *reversed {
        segments.iter().rev().all(&mut visit)
    } else {
        segments.iter().all(&mut visit)
    }
}

fn planar_points(points: &[Point3], tolerance: f64) -> bool {
    if points.len() <= 2 {
        return true;
    }
    // Newell's normal is robust for a sampled curve, including a nearly closed loop.
    let mut normal = [0.0f64; 3];
    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let length = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !length.is_finite() || length <= 1.0e-15 {
        return true;
    }
    let unit = [normal[0] / length, normal[1] / length, normal[2] / length];
    let origin = points[0];
    points.iter().all(|point| {
        let offset = [
            point[0] - origin[0],
            point[1] - origin[1],
            point[2] - origin[2],
        ];
        (offset[0] * unit[0] + offset[1] * unit[1] + offset[2] * unit[2]).abs() <= tolerance
    })
}

/// Classify a curve and measure closedness/planarity under `tolerance`.
///
/// Policy: a curve is closed when the distance between its sampled endpoints is at most the stated
/// tolerance. A curve whose sampled length is at or below that tolerance is refused as degenerate;
/// it is never silently reclassified as a point or polygon.
pub fn classify_curve(curve: &Curve, tolerance: f64, limits: &Limits) -> Result<CurveSemantics> {
    curve.validate(limits)?;
    check_tolerance(tolerance)?;
    let form = match curve {
        Curve::Line { .. } => CurveForm::Line,
        Curve::Circle { .. } => CurveForm::Circle,
        Curve::Nurbs(_) => CurveForm::Nurbs,
        Curve::Polycurve { .. } if is_line_polyline(curve, tolerance) => CurveForm::Polyline,
        Curve::Polycurve { .. } => CurveForm::Polycurve,
    };
    let points = surface_kernel::sample_curve(curve, 33, limits)?;
    let first = *points.first().ok_or(Error::Invalid("curve samples"))?;
    let last = *points.last().ok_or(Error::Invalid("curve samples"))?;
    let mut length = 0.0;
    for pair in points.windows(2) {
        length += distance(pair[0], pair[1]);
    }
    if !length.is_finite() || length <= tolerance {
        return Err(Error::Invalid("degenerate curve within tolerance"));
    }
    Ok(CurveSemantics {
        form,
        closed: distance(first, last) <= tolerance,
        planar: planar_points(&points, tolerance),
        endpoints: [first, last],
        tolerance,
    })
}

/// NURBS curve cardinality and rationality, checked before any kernel conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NurbsCurveSemantics {
    pub degree: usize,
    pub pole_count: usize,
    pub weight_count: usize,
    pub knot_count: usize,
    pub expected_knot_count: usize,
    pub rational: bool,
    pub closed: bool,
    pub planar: bool,
}

/// Inspect a NURBS curve. Knot cardinality (`count + degree + 1`) and positive weights are enforced
/// by the existing record validation, so a mismatched curve is refused before OCCT sees it.
pub fn inspect_nurbs_curve(
    curve: &NurbsCurve,
    tolerance: f64,
    limits: &Limits,
) -> Result<NurbsCurveSemantics> {
    curve.validate(limits)?;
    let measured = classify_curve(&Curve::Nurbs(curve.clone()), tolerance, limits)?;
    let expected_knot_count = curve
        .poles
        .len()
        .checked_add(curve.degree)
        .and_then(|n| n.checked_add(1))
        .ok_or(Error::Overflow("knot count"))?;
    Ok(NurbsCurveSemantics {
        degree: curve.degree,
        pole_count: curve.poles.len(),
        weight_count: curve.weights.len(),
        knot_count: curve.knots.len(),
        expected_knot_count,
        rational: curve
            .weights
            .iter()
            .any(|weight| (*weight - 1.0).abs() > 0.0),
        closed: measured.closed,
        planar: measured.planar,
    })
}

/* -------------------------------- surfaces -------------------------------- */

/// U/V periodicity and orientation a caller expects for a parametric surface. A swapped U/V record
/// is reported as an orientation error instead of being silently reinterpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceExpectation {
    pub periodic: [bool; 2],
    pub v_reversed: bool,
}

/// Surface domain, seam and periodicity with the input values preserved verbatim.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceSemantics {
    pub domain: [[f64; 2]; 2],
    pub periodic: [bool; 2],
    /// Seam parameter per axis when the axis is periodic or geometrically closed.
    pub seam: [Option<f64>; 2],
    pub v_reversed: bool,
}

fn axis_closed(surface: &NurbsSurface, axis: usize, tolerance: f64) -> bool {
    if axis == 0 {
        for v in 0..surface.size[1] {
            let first = surface.poles[v];
            let last = surface.poles[(surface.size[0] - 1) * surface.size[1] + v];
            if distance(first, last) > tolerance {
                return false;
            }
        }
    } else {
        for u in 0..surface.size[0] {
            let first = surface.poles[u * surface.size[1]];
            let last = surface.poles[u * surface.size[1] + surface.size[1] - 1];
            if distance(first, last) > tolerance {
                return false;
            }
        }
    }
    true
}

fn unclamped_knots(knots: &[f64], degree: usize) -> bool {
    if knots.len() < degree + 1 {
        return false;
    }
    let first_clamped = knots[..=degree].windows(2).all(|pair| pair[0] == pair[1]);
    let tail = knots.len().saturating_sub(degree + 1);
    let last_clamped = knots[tail..].windows(2).all(|pair| pair[0] == pair[1]);
    !(first_clamped && last_clamped)
}

/// Classify a NURBS surface.
///
/// Rules:
/// - the record validates first;
/// - the stored `periodic` flag must agree with the knot vector's clamped/unclamped shape; a
///   periodic record with clamped knots is `periodic flag mismatch`, and an unclamped record with
///   `periodic = false` is `surface periodic flag missing`;
/// - a geometrically closed axis (coincident seam rows/columns within tolerance) reports a seam at
///   the domain start even when the surface is closed but not periodic (Rhino distinguishes them).
pub fn classify_surface(
    surface: &NurbsSurface,
    tolerance: f64,
    limits: &Limits,
) -> Result<SurfaceSemantics> {
    surface.validate(limits)?;
    check_tolerance(tolerance)?;
    let closed = [
        axis_closed(surface, 0, tolerance),
        axis_closed(surface, 1, tolerance),
    ];
    let knots_periodic = [
        unclamped_knots(&surface.knots[0], surface.degree[0]),
        unclamped_knots(&surface.knots[1], surface.degree[1]),
    ];
    for (axis, knots_axis_periodic) in knots_periodic.iter().enumerate() {
        if surface.periodic[axis] != *knots_axis_periodic {
            return Err(if surface.periodic[axis] {
                Error::Invalid("surface periodic flag mismatch")
            } else {
                Error::Invalid("surface periodic flag missing")
            });
        }
    }
    let seam = [
        if surface.periodic[0] || closed[0] {
            Some(surface.domain[0][0])
        } else {
            None
        },
        if surface.periodic[1] || closed[1] {
            Some(surface.domain[1][0])
        } else {
            None
        },
    ];
    Ok(SurfaceSemantics {
        domain: surface.domain,
        periodic: surface.periodic,
        seam,
        v_reversed: surface.reversed,
    })
}

impl SurfaceSemantics {
    /// Refuse a swapped/mis-mapped surface. The caller states the intended periodic axes and V
    /// orientation (for example, a cylinder is U-periodic and V-forward).
    pub fn require_orientation(&self, expected: SurfaceExpectation) -> Result<()> {
        if self.periodic != expected.periodic || self.v_reversed != expected.v_reversed {
            return Err(Error::Invalid("surface U/V orientation"));
        }
        Ok(())
    }
}

/* ---------------------------- trimmed surface ---------------------------- */

fn point_in_polygon(point: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let (xi, yi) = (polygon[i][0], polygon[i][1]);
        let (xj, yj) = (polygon[j][0], polygon[j][1]);
        if (yi > point[1]) != (yj > point[1])
            && point[0] < (xj - xi) * (point[1] - yi) / (yj - yi) + xi
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn polygon_area(polygon: &[[f64; 2]]) -> f64 {
    let mut twice = 0.0;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        twice += polygon[j][0] * polygon[i][1] - polygon[i][0] * polygon[j][1];
        j = i;
    }
    twice / 2.0
}

fn validate_loop(points: &[[f64; 2]], domain: [[f64; 2]; 2], label: &'static str) -> Result<()> {
    if points.len() < 3 {
        return Err(Error::Invalid(label));
    }
    for point in points {
        if !point[0].is_finite() || !point[1].is_finite() {
            return Err(Error::Invalid(label));
        }
        if point[0] < domain[0][0]
            || point[0] > domain[0][1]
            || point[1] < domain[1][0]
            || point[1] > domain[1][1]
        {
            return Err(Error::Invalid("trim loop outside surface domain"));
        }
    }
    for pair in points.windows(2) {
        if pair[0] == pair[1] {
            return Err(Error::Invalid("trim loop duplicate point"));
        }
    }
    if points.first() == points.last() {
        return Err(Error::Invalid("trim loop repeats its first point"));
    }
    let area = polygon_area(points);
    if !area.is_finite() || area.abs() <= 1.0e-12 {
        return Err(Error::Invalid("trim loop area"));
    }
    Ok(())
}

/// The underlying surface plus its outer and inner UV trim loops. The surface is never replaced by
/// the trim polygons, and the record never exposes an "untrimmed visible face".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrimmedSurface {
    pub underlying: NurbsSurface,
    pub outer: Vec<[f64; 2]>,
    pub inners: Vec<Vec<[f64; 2]>>,
    pub reversed: bool,
}

impl TrimmedSurface {
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        self.underlying.validate(limits)?;
        let domain = self.underlying.domain;
        validate_loop(&self.outer, domain, "trim outer loop")?;
        for inner in &self.inners {
            validate_loop(inner, domain, "trim inner loop")?;
            for point in inner {
                if !point_in_polygon(*point, &self.outer) {
                    return Err(Error::Invalid("trim inner loop outside outer loop"));
                }
            }
        }
        Ok(())
    }

    pub fn is_trimmed(&self) -> bool {
        true
    }

    pub fn underlying_surface(&self) -> &NurbsSurface {
        &self.underlying
    }

    /// A trimmed patch is never promoted to the full untrimmed surface. The underlying surface is
    /// available for evaluation, but no API produces a visible untrimmed `Geometry::Surface`.
    pub fn to_untrimmed(&self) -> Result<Geometry> {
        Err(Error::Unsupported(
            "trimmed surface cannot be promoted to an untrimmed surface",
        ))
    }

    pub fn to_extension(&self) -> Result<Extension> {
        Extension::new(
            1,
            serde_json::to_string(self).map_err(|error| Error::Io(error.to_string()))?,
        )
    }

    pub fn from_extension(extension: &Extension, limits: &Limits) -> Result<Self> {
        let trimmed: Self = serde_json::from_str(&extension.payload)
            .map_err(|_| Error::Invalid("trimmed surface payload"))?;
        trimmed.validate(limits)?;
        Ok(trimmed)
    }
}

pub fn trimmed_surface_of(entity: &Entity, limits: &Limits) -> Result<Option<TrimmedSurface>> {
    match entity.extensions.get(TRIM_SURFACE_EXTENSION_SLOT) {
        Some(extension) => Ok(Some(TrimmedSurface::from_extension(extension, limits)?)),
        None => Ok(None),
    }
}

pub fn set_trimmed_surface(
    entity: &mut Entity,
    trimmed: &TrimmedSurface,
    limits: &Limits,
) -> Result<()> {
    trimmed.validate(limits)?;
    match &entity.geometry {
        Geometry::Surface(surface) if surface == &trimmed.underlying => {}
        Geometry::Surface(_) => return Err(Error::Invalid("trimmed surface underlying mismatch")),
        _ => return Err(Error::Invalid("trimmed surface requires surface geometry")),
    }
    entity.extensions.insert(
        TRIM_SURFACE_EXTENSION_SLOT.to_string(),
        trimmed.to_extension()?,
    );
    Ok(())
}

pub fn clear_trimmed_surface(entity: &mut Entity) {
    entity.extensions.remove(TRIM_SURFACE_EXTENSION_SLOT);
}

/* ---------------------------- display isocurve ---------------------------- */

/// An exact iso-curve of a surface that exists only for display. It is not a model edge, is not
/// pickable, and cannot be converted into persistent `Geometry`.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayIsocurve {
    curve: NurbsCurve,
    iso: SurfaceIso,
    value: f64,
}

impl DisplayIsocurve {
    pub fn from_surface(
        surface: &NurbsSurface,
        iso: SurfaceIso,
        value: f64,
        limits: &Limits,
    ) -> Result<Self> {
        let curve = surface_kernel::isocurve(surface, iso, value, limits)?;
        Ok(Self { curve, iso, value })
    }

    pub fn curve(&self) -> &NurbsCurve {
        &self.curve
    }

    pub fn iso(&self) -> SurfaceIso {
        self.iso
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn is_display_only(&self) -> bool {
        true
    }

    pub fn is_model_edge(&self) -> bool {
        false
    }

    pub fn is_pickable(&self) -> bool {
        false
    }

    /// A display isocurve can never become persistent geometry or an exported edge.
    pub fn to_geometry(&self) -> Result<Geometry> {
        Err(Error::Unsupported("isocurve is display-only"))
    }
}

/// A boundary edge of a B-Rep display snapshot: real topology, pickable, and distinct from a
/// display-only isocurve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryEdge {
    pub object: ObjectId,
    pub face: Uuid,
    pub edge: Uuid,
}

impl BoundaryEdge {
    pub fn new(object: ObjectId, face: Uuid, edge: Uuid) -> Result<Self> {
        if object.is_nil() || face.is_nil() || edge.is_nil() {
            return Err(Error::Invalid("boundary edge identity"));
        }
        Ok(Self { object, face, edge })
    }

    pub fn is_model_edge(&self) -> bool {
        true
    }

    pub fn is_pickable(&self) -> bool {
        true
    }

    pub fn is_display_only(&self) -> bool {
        false
    }
}

/* ------------------------------ solid / shell ------------------------------ */

/// The portable shape class after topology is checked. `Surface`/`Polysurface` are open shells;
/// only a fully closed shell may claim `Solid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeClass {
    Surface,
    Polysurface,
    Solid,
}

fn face_signature(body: &BrepBody, face: &crate::brep_kernel::Face) -> Vec<[u64; 3]> {
    let mut points = Vec::new();
    for wire_index in std::iter::once(face.outer).chain(face.inners.iter().copied()) {
        let wire = &body.wires[wire_index as usize];
        for edge in &wire.edges {
            let edge = &body.edges[*edge as usize];
            for vertex in [edge.start, edge.end] {
                points.push(body.vertices[vertex as usize].point.map(f64::to_bits));
            }
        }
    }
    points.sort_unstable();
    points
}

/// Classify a B-Rep body: a closed shell is a solid, an open one-face shell is a surface, and an
/// open multi-face shell is a polysurface. A body that claims `Solid` but is open is refused, and a
/// shell with two coincident faces is refused as self-overlapping.
pub fn classify_shell(body: &BrepBody, limits: &Limits) -> Result<ShapeClass> {
    body.validate(limits)?;
    if body.shells.is_empty() || body.faces.is_empty() {
        return Err(Error::Invalid("brep body has no shell"));
    }
    let mut faces = BTreeSet::new();
    let mut face_count = 0usize;
    for shell in &body.shells {
        for face_index in &shell.faces {
            let face = &body.faces[*face_index as usize];
            if !faces.insert(face_signature(body, face)) {
                return Err(Error::Invalid("self-overlapping shell"));
            }
            face_count += 1;
        }
    }
    let closed = body.shells.iter().all(|shell| shell.closed);
    if body.kind == ShapeKind::Solid && !closed {
        return Err(Error::Invalid("open shell is not a solid"));
    }
    if closed {
        if !matches!(body.kind, ShapeKind::Solid | ShapeKind::Shell) {
            return Err(Error::Invalid("closed shell kind"));
        }
        return Ok(ShapeClass::Solid);
    }
    if face_count <= 1 {
        Ok(ShapeClass::Surface)
    } else {
        Ok(ShapeClass::Polysurface)
    }
}

/* -------------------------------- extrusion -------------------------------- */

/// A lightweight extrusion: a closed planar polyline profile, a direction and a height. The profile
/// stays the entity geometry; conversion to a portable B-Rep body happens on demand and keeps the
/// source record as provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extrusion {
    pub profile: Curve,
    pub direction: Point3,
    pub height: f64,
    pub capped: bool,
}

/// The converted body plus the exact extrusion it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtrusionConversion {
    pub source: Extrusion,
    pub body: BrepBody,
}

impl Extrusion {
    pub fn validate(&self, tolerance: f64, limits: &Limits) -> Result<()> {
        check_tolerance(tolerance)?;
        self.profile.validate(limits)?;
        let semantics = classify_curve(&self.profile, tolerance, limits)?;
        if !semantics.closed {
            return Err(Error::Invalid("extrusion profile must be closed"));
        }
        if !semantics.planar {
            return Err(Error::Invalid("extrusion profile must be planar"));
        }
        if !self.direction.iter().all(|value| value.is_finite()) {
            return Err(Error::Invalid("extrusion direction"));
        }
        let length = self.direction[0]
            .hypot(self.direction[1])
            .hypot(self.direction[2]);
        if !length.is_finite() || length <= 1.0e-9 {
            return Err(Error::Invalid("extrusion direction"));
        }
        if !self.height.is_finite() || self.height == 0.0 {
            return Err(Error::Invalid("extrusion height"));
        }
        if !self.capped {
            return Err(Error::Unsupported("open extrusion is not a solid"));
        }
        Ok(())
    }

    fn loop_points(&self, tolerance: f64, limits: &Limits) -> Result<Vec<Point3>> {
        let Curve::Polycurve { segments, reversed } = &self.profile else {
            return Err(Error::Unsupported(
                "extrusion profile requires a closed polyline",
            ));
        };
        if segments.is_empty() {
            return Err(Error::Invalid("extrusion profile segments"));
        }
        let mut points = Vec::new();
        let mut push = |point: Point3| -> Result<()> {
            if points
                .last()
                .is_none_or(|last| distance(*last, point) > tolerance)
            {
                points.push(point);
            }
            Ok(())
        };
        let mut visit = |segment: &Curve| -> Result<()> {
            let Curve::Line { start, end, .. } = segment else {
                return Err(Error::Unsupported(
                    "extrusion profile requires line segments",
                ));
            };
            push(*start)?;
            push(*end)?;
            Ok(())
        };
        if *reversed {
            for segment in segments.iter().rev() {
                visit(segment)?;
            }
        } else {
            for segment in segments {
                visit(segment)?;
            }
        }
        if points.len() > 3 && distance(points[0], *points.last().unwrap()) <= tolerance {
            points.pop();
        }
        if points.len() < 3 {
            return Err(Error::Invalid("extrusion profile loop"));
        }
        curve_build::polyline(&points)?;
        let _ = limits;
        Ok(points)
    }

    /// Convert the lightweight primitive to a closed, portable B-Rep body. Each face owns its own
    /// directed edges, so the portable body is a set of closed face loops rather than a sewn shape;
    /// the kernel adapter sews on submission.
    pub fn to_brep_body(&self, tolerance: f64, limits: &Limits) -> Result<BrepBody> {
        self.validate(tolerance, limits)?;
        let points = self.loop_points(tolerance, limits)?;
        let length = self.direction[0]
            .hypot(self.direction[1])
            .hypot(self.direction[2]);
        let unit = [
            self.direction[0] / length,
            self.direction[1] / length,
            self.direction[2] / length,
        ];
        let count = points.len();
        let mut vertices = Vec::with_capacity(count * 2);
        for point in &points {
            vertices.push(crate::brep_kernel::Vertex {
                point: *point,
                tolerance,
            });
        }
        for point in &points {
            vertices.push(crate::brep_kernel::Vertex {
                point: [
                    point[0] + unit[0] * self.height,
                    point[1] + unit[1] * self.height,
                    point[2] + unit[2] * self.height,
                ],
                tolerance,
            });
        }
        let mut edges = Vec::new();
        let mut wires = Vec::new();
        let mut faces = Vec::new();
        let edge = |edges: &mut Vec<crate::brep_kernel::Edge>, start: u32, end: u32| -> u32 {
            let index = edges.len() as u32;
            edges.push(crate::brep_kernel::Edge {
                start,
                end,
                tolerance,
                closed: false,
            });
            index
        };
        // Bottom cap.
        let mut bottom = Vec::with_capacity(count);
        for i in 0..count {
            bottom.push(edge(&mut edges, i as u32, ((i + 1) % count) as u32));
        }
        wires.push(crate::brep_kernel::Wire {
            edges: bottom,
            closed: true,
        });
        faces.push(crate::brep_kernel::Face {
            outer: 0,
            inners: vec![],
            tolerance,
        });
        // Top cap (same winding, distinct edge records).
        let mut top = Vec::with_capacity(count);
        for i in 0..count {
            top.push(edge(
                &mut edges,
                (count + i) as u32,
                (count + (i + 1) % count) as u32,
            ));
        }
        wires.push(crate::brep_kernel::Wire {
            edges: top,
            closed: true,
        });
        faces.push(crate::brep_kernel::Face {
            outer: 1,
            inners: vec![],
            tolerance,
        });
        // Side faces, each with its own four directed edges.
        for i in 0..count {
            let next = (i + 1) % count;
            let a = edge(&mut edges, i as u32, next as u32);
            let b = edge(&mut edges, next as u32, (count + next) as u32);
            let c = edge(&mut edges, (count + next) as u32, (count + i) as u32);
            let d = edge(&mut edges, (count + i) as u32, i as u32);
            let wire = wires.len() as u32;
            wires.push(crate::brep_kernel::Wire {
                edges: vec![a, b, c, d],
                closed: true,
            });
            faces.push(crate::brep_kernel::Face {
                outer: wire,
                inners: vec![],
                tolerance,
            });
        }
        let body = BrepBody {
            vertices,
            edges,
            wires,
            faces,
            shells: vec![crate::brep_kernel::Shell {
                faces: (0..count as u32 + 2).collect(),
                closed: true,
            }],
            kind: ShapeKind::Solid,
            tolerance,
        };
        body.validate(limits)?;
        Ok(body)
    }

    pub fn convert(&self, tolerance: f64, limits: &Limits) -> Result<ExtrusionConversion> {
        Ok(ExtrusionConversion {
            source: self.clone(),
            body: self.to_brep_body(tolerance, limits)?,
        })
    }

    pub fn to_extension(&self) -> Result<Extension> {
        Extension::new(
            1,
            serde_json::to_string(self).map_err(|error| Error::Io(error.to_string()))?,
        )
    }

    pub fn from_extension(extension: &Extension, tolerance: f64, limits: &Limits) -> Result<Self> {
        let extrusion: Self = serde_json::from_str(&extension.payload)
            .map_err(|_| Error::Invalid("extrusion payload"))?;
        extrusion.validate(tolerance, limits)?;
        Ok(extrusion)
    }
}

pub fn extrusion_of(entity: &Entity, limits: &Limits) -> Result<Option<Extrusion>> {
    match entity.extensions.get(EXTRUSION_EXTENSION_SLOT) {
        Some(extension) => Ok(Some(Extrusion::from_extension(extension, 1.0e-6, limits)?)),
        None => Ok(None),
    }
}

pub fn set_extrusion(entity: &mut Entity, extrusion: &Extrusion, limits: &Limits) -> Result<()> {
    extrusion.validate(1.0e-6, limits)?;
    match &entity.geometry {
        Geometry::Curve(curve) if curve == &extrusion.profile => {}
        Geometry::Curve(_) => return Err(Error::Invalid("extrusion profile mismatch")),
        _ => return Err(Error::Invalid("extrusion requires profile geometry")),
    }
    entity.extensions.insert(
        EXTRUSION_EXTENSION_SLOT.to_string(),
        extrusion.to_extension()?,
    );
    Ok(())
}

pub fn clear_extrusion(entity: &mut Entity) {
    entity.extensions.remove(EXTRUSION_EXTENSION_SLOT);
}

/* --------------------------- mesh vs exact NURBS --------------------------- */

/// Exact geometry kinds. A mesh or point cloud is never exact NURBS/B-Rep.
pub fn is_exact_geometry(geometry: &Geometry) -> bool {
    matches!(
        geometry,
        Geometry::Point(_)
            | Geometry::Curve(_)
            | Geometry::Surface(_)
            | Geometry::Brep(_)
            | Geometry::Instance(_)
    )
}

/// Refuse to treat a display mesh as exact geometry. There is deliberately no implicit mesh-to-NURBS
/// conversion; a caller that needs one must run an explicit refit/reconstruction with its own oracle.
pub fn require_exact_nurbs(geometry: &Geometry) -> Result<()> {
    if is_exact_geometry(geometry) {
        Ok(())
    } else {
        Err(Error::Unsupported("mesh is not exact NURBS"))
    }
}

/* -------------------------------- instances -------------------------------- */

/// One block occurrence: a distinct object identity, its shared definition and its own transform.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceSemantic {
    pub object: ObjectId,
    pub definition: DefinitionId,
    pub transform: Transform,
}

/// Every live occurrence of one definition, in stable object-id order.
pub fn block_instances(
    document: &Document,
    definition: DefinitionId,
) -> Result<Vec<InstanceSemantic>> {
    if !document.definitions.contains_key(&definition) {
        return Err(Error::Missing("instance definition"));
    }
    let mut instances = Vec::new();
    for (id, entity) in &document.entities {
        if let Geometry::Instance(instance_definition) = entity.geometry
            && instance_definition == definition
        {
            instances.push(InstanceSemantic {
                object: *id,
                definition,
                transform: entity.placement,
            });
        }
    }
    Ok(instances)
}

/// Change one occurrence transform. The shared definition is never touched.
pub fn set_instance_transform(
    document: &mut Document,
    object: ObjectId,
    transform: Transform,
) -> Result<()> {
    transform.validate()?;
    let entity = document
        .entities
        .get_mut(&object)
        .ok_or(Error::Missing("instance object"))?;
    if !matches!(entity.geometry, Geometry::Instance(_)) {
        return Err(Error::Invalid("not an instance"));
    }
    entity.placement = transform;
    entity.geometry_revision = entity
        .geometry_revision
        .checked_add(1)
        .ok_or(Error::Overflow("geometry revision"))?;
    entity.generation = entity
        .generation
        .checked_add(1)
        .ok_or(Error::Overflow("entity generation"))?;
    Ok(())
}

/* ----------------------------- model validation ----------------------------- */

/// Validate a known semantic extension payload during document validation. Unknown slots stay
/// opaque text and are preserved by the archive.
pub fn validate_extension(slot: &str, payload: &str, limits: &Limits) -> Result<()> {
    let extension = crate::model::Extension {
        version: 1,
        payload: payload.to_string(),
    };
    match slot {
        TRIM_SURFACE_EXTENSION_SLOT => {
            TrimmedSurface::from_extension(&extension, limits)?;
        }
        EXTRUSION_EXTENSION_SLOT => {
            Extrusion::from_extension(&extension, 1.0e-6, limits)?;
        }
        _ => {}
    }
    Ok(())
}

/// Evaluate one curve endpoint under the same parameter rules as the renderer.
pub fn curve_endpoints(curve: &Curve, limits: &Limits) -> Result<[Point3; 2]> {
    curve.validate(limits)?;
    let start = evaluate_curve(
        curve,
        Parameter::Fraction(0.0),
        0,
        Transform::identity(),
        limits,
    )?;
    let end = evaluate_curve(
        curve,
        Parameter::Fraction(1.0),
        0,
        Transform::identity(),
        limits,
    )?;
    Ok([
        *start.first().ok_or(Error::Invalid("curve endpoint"))?,
        *end.first().ok_or(Error::Invalid("curve endpoint"))?,
    ])
}
