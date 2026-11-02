// SPDX-License-Identifier: LGPL-2.1-or-later
//! Nonlinear deformers: bend, taper, twist, shear, maelstrom and soft move.
//!
//! Matrix applies these as space deformations, and a nonlinear map cannot be stored in a
//! placement, so a deformed object gets new geometry: the control points of curves and
//! surfaces and the vertices of meshes and clouds are mapped directly. A straight line stops
//! being straight under a bend, a twist or a maelstrom, so it becomes a polycurve of the
//! mapped samples; under a shear the map is affine and the line stays a line.
//!
//! A solid is refused. Deforming a B-Rep correctly needs a kernel deformer; mapping only its
//! display mesh would look deformed while the solid stayed wrong, and this module does not
//! report that as success.
//!
//! Every deform is described by a base point and a unit axis taken from the command's typed
//! points, so the same map is reproducible from the record and from a test.
use crate::budget::Limits;
use crate::curve_ops;
use crate::error::{Error, Result};
use crate::geometry::{Curve, Geometry, Mesh, NurbsSurface, Point3, Transform};

/// The number of samples a nonlinear map puts on a curve segment. Seventeen points keep a
/// quarter-turn bend within a fraction of a percent of the exact arc while staying cheap, and
/// the value is fixed so a record names the same geometry a test does.
pub const LINE_SAMPLES: usize = 17;

fn finite(point: Point3, name: &'static str) -> Result<()> {
    if point.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(Error::Invalid(name))
    }
}

fn unit(vector: Point3, name: &'static str) -> Result<Point3> {
    finite(vector, name)?;
    let length = dot(vector, vector).sqrt();
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid(name));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn mul(vector: Point3, factor: f64) -> Point3 {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

/// The component of `vector` along the unit `axis`, and what is left of it.
fn split(vector: Point3, axis: Point3) -> (f64, Point3) {
    let along = dot(vector, axis);
    (along, sub(vector, mul(axis, along)))
}

/// Rotate `vector` about the unit `axis` by `radians` (Rodrigues).
fn rotate(vector: Point3, axis: Point3, radians: f64) -> Point3 {
    let (sine, cosine) = radians.sin_cos();
    let term = add(mul(vector, cosine), mul(cross(axis, vector), sine));
    add(term, mul(axis, dot(axis, vector) * (1.0 - cosine)))
}

/// The affine transform of a shear: `p -> p + factor * ((p - base) . measure) * axis`.
fn shear_transform(base: Point3, measure: Point3, axis: Point3, factor: f64) -> Transform {
    let mut matrix = Transform::identity().0;
    for row in 0..3 {
        for column in 0..3 {
            matrix[row][column] += factor * axis[row] * measure[column];
        }
    }
    let offset = factor * dot(base, measure);
    for row in 0..3 {
        matrix[row][3] = -offset * axis[row];
    }
    Transform(matrix)
}

/// One documented nonlinear deformation.
#[derive(Debug, Clone, PartialEq)]
pub enum Deform {
    /// Bend the span along `axis` into an arc of `angle_degrees`, the arc lying in the plane
    /// of `axis` and `bend` (a direction that must not be parallel to `axis`). A positive
    /// angle bends towards `bend`.
    Bend {
        base: Point3,
        axis: Point3,
        bend: Point3,
        angle_degrees: f64,
    },
    /// Scale the cross-section perpendicular to `axis` by `factor`, measured over the span:
    /// `1` at `base` and `factor` at the far end.
    Taper {
        base: Point3,
        axis: Point3,
        factor: f64,
    },
    /// Turn the cross-section about `axis` by `angle_degrees` over the span: `0` at `base`
    /// and the full angle at the far end.
    Twist {
        base: Point3,
        axis: Point3,
        angle_degrees: f64,
    },
    /// Affine shear: the component measured along `measure` moves the point by
    /// `factor * that component` along `axis`. Both directions are normalised.
    Shear {
        base: Point3,
        measure: Point3,
        axis: Point3,
        factor: f64,
    },
    /// A twist plus a radial pull: the cross-section turns by `angle_degrees` and shrinks by
    /// `pull` (0..1) over the span.
    Maelstrom {
        base: Point3,
        axis: Point3,
        angle_degrees: f64,
        pull: f64,
    },
    /// Move points by `displacement` with a smooth falloff inside `radius` of `base`.
    SoftMove {
        base: Point3,
        displacement: Point3,
        radius: f64,
    },
}

impl Deform {
    /// The direction the span is measured along, when the deform has one.
    pub fn axis(&self) -> Option<Point3> {
        match self {
            Self::Bend { axis, .. }
            | Self::Taper { axis, .. }
            | Self::Twist { axis, .. }
            | Self::Maelstrom { axis, .. } => Some(*axis),
            Self::Shear { measure, .. } => Some(*measure),
            Self::SoftMove { .. } => None,
        }
    }

    pub fn base(&self) -> Point3 {
        match self {
            Self::Bend { base, .. }
            | Self::Taper { base, .. }
            | Self::Twist { base, .. }
            | Self::Shear { base, .. }
            | Self::Maelstrom { base, .. }
            | Self::SoftMove { base, .. } => *base,
        }
    }

    /// A shear keeps every straight segment straight, so a line need not be sampled.
    pub fn is_affine(&self) -> bool {
        matches!(self, Self::Shear { .. })
    }

    /// The shear as the affine transform it is.
    pub fn affine(&self) -> Option<Transform> {
        match self {
            Self::Shear {
                base,
                measure,
                axis,
                factor,
            } => Some(shear_transform(*base, *measure, *axis, *factor)),
            _ => None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Bend {
                base,
                axis,
                bend,
                angle_degrees,
            } => {
                finite(*base, "bend base point")?;
                let axis = unit(*axis, "bend axis")?;
                let bend = unit(*bend, "bend direction")?;
                if !angle_degrees.is_finite() {
                    return Err(Error::Invalid("bend angle"));
                }
                let (_, perpendicular) = split(bend, axis);
                if dot(perpendicular, perpendicular).sqrt() <= 1.0e-9 {
                    return Err(Error::Invalid("bend direction is the axis"));
                }
                Ok(())
            }
            Self::Taper { base, axis, factor } => {
                finite(*base, "taper base point")?;
                unit(*axis, "taper axis")?;
                if !factor.is_finite() || *factor <= 0.0 {
                    return Err(Error::Invalid("taper factor"));
                }
                Ok(())
            }
            Self::Twist {
                base,
                axis,
                angle_degrees,
            } => {
                finite(*base, "twist base point")?;
                unit(*axis, "twist axis")?;
                if !angle_degrees.is_finite() {
                    return Err(Error::Invalid("twist angle"));
                }
                Ok(())
            }
            Self::Shear {
                base,
                measure,
                axis,
                factor,
            } => {
                finite(*base, "shear base point")?;
                let measure = unit(*measure, "shear measure direction")?;
                let axis = unit(*axis, "shear direction")?;
                if !factor.is_finite() {
                    return Err(Error::Invalid("shear factor"));
                }
                if cross(measure, axis)
                    .iter()
                    .all(|value| value.abs() <= 1.0e-9)
                {
                    return Err(Error::Invalid("shear directions are parallel"));
                }
                Ok(())
            }
            Self::Maelstrom {
                base,
                axis,
                angle_degrees,
                pull,
            } => {
                finite(*base, "maelstrom base point")?;
                unit(*axis, "maelstrom axis")?;
                if !angle_degrees.is_finite() {
                    return Err(Error::Invalid("maelstrom angle"));
                }
                if !pull.is_finite() || !(0.0..1.0).contains(pull) {
                    return Err(Error::Invalid("maelstrom pull"));
                }
                Ok(())
            }
            Self::SoftMove {
                base,
                displacement,
                radius,
            } => {
                finite(*base, "soft move base point")?;
                finite(*displacement, "soft move displacement")?;
                if !radius.is_finite() || *radius <= 0.0 {
                    return Err(Error::Invalid("soft move radius"));
                }
                if dot(*displacement, *displacement) <= 0.0 {
                    return Err(Error::Invalid("soft move displacement"));
                }
                Ok(())
            }
        }
    }

    /// The span the deform is measured over: the extent of the geometry along the axis, or
    /// the falloff radius for a soft move.
    pub fn span(&self, geometry: &Geometry, limits: &Limits) -> Result<f64> {
        match self {
            Self::SoftMove { radius, .. } => Ok(*radius),
            _ => {
                let axis = unit(
                    self.axis().ok_or(Error::Missing("deform axis"))?,
                    "deform axis",
                )?;
                let base = self.base();
                let (low, high) = extents(geometry, base, axis, limits)?;
                let span = high - low;
                if !span.is_finite() || span <= 1.0e-9 {
                    return Err(Error::Invalid("deformed span"));
                }
                Ok(span)
            }
        }
    }

    /// Map one point. `span` is the extent along the axis from [`Self::span`].
    pub fn map(&self, point: Point3, span: f64) -> Result<Point3> {
        finite(point, "deformed point")?;
        if !span.is_finite() || span <= 0.0 {
            return Err(Error::Invalid("deformed span"));
        }
        match self {
            Self::Bend {
                base,
                axis,
                bend,
                angle_degrees,
            } => {
                let axis = unit(*axis, "bend axis")?;
                let bend = unit(*bend, "bend direction")?;
                let (_, perpendicular) = split(bend, axis);
                let normal = unit(perpendicular, "bend direction")?;
                let offset = sub(point, *base);
                let (distance, rest) = split(offset, axis);
                let height = dot(rest, normal);
                let cross_section = sub(rest, mul(normal, height));
                let total = angle_degrees.to_radians();
                if total.abs() <= 1.0e-12 {
                    return Ok(point);
                }
                let curvature = total / span;
                let radius = 1.0 / curvature;
                let sine = (curvature * distance).sin();
                let cosine = (curvature * distance).cos();
                let arc = add(
                    mul(axis, (radius - height) * sine),
                    mul(normal, radius - (radius - height) * cosine),
                );
                Ok(add(add(*base, arc), cross_section))
            }
            Self::Taper { base, axis, factor } => {
                let axis = unit(*axis, "taper axis")?;
                let (distance, rest) = split(sub(point, *base), axis);
                let share = (distance / span).clamp(0.0, 1.0);
                let scale = 1.0 + (factor - 1.0) * share;
                Ok(add(add(*base, mul(axis, distance)), mul(rest, scale)))
            }
            Self::Twist {
                base,
                axis,
                angle_degrees,
            } => {
                let axis = unit(*axis, "twist axis")?;
                let (distance, rest) = split(sub(point, *base), axis);
                let share = (distance / span).clamp(0.0, 1.0);
                let turn = angle_degrees.to_radians() * share;
                Ok(add(
                    add(*base, mul(axis, distance)),
                    rotate(rest, axis, turn),
                ))
            }
            Self::Shear {
                base,
                measure,
                axis,
                factor,
            } => {
                let measure = unit(*measure, "shear measure direction")?;
                let axis = unit(*axis, "shear direction")?;
                let along = dot(sub(point, *base), measure);
                Ok(add(point, mul(axis, factor * along)))
            }
            Self::Maelstrom {
                base,
                axis,
                angle_degrees,
                pull,
            } => {
                let axis = unit(*axis, "maelstrom axis")?;
                let (distance, rest) = split(sub(point, *base), axis);
                let share = (distance / span).clamp(0.0, 1.0);
                let turn = angle_degrees.to_radians() * share;
                let shrink = 1.0 - pull * share;
                Ok(add(
                    add(*base, mul(axis, distance)),
                    mul(rotate(rest, axis, turn), shrink),
                ))
            }
            Self::SoftMove {
                base,
                displacement,
                radius,
            } => {
                let offset = sub(point, *base);
                let distance = dot(offset, offset).sqrt();
                if distance >= *radius {
                    return Ok(point);
                }
                if distance <= 1.0e-12 {
                    return Ok(add(point, *displacement));
                }
                let share = distance / radius;
                // At the base the weight is 1 and the tangent is continuous at the radius,
                // so the moved region joins the untouched geometry without a crease.
                let squared = share * share;
                let weight = (1.0 - squared) * (1.0 - squared);
                Ok(add(point, mul(*displacement, weight)))
            }
        }
    }
}

/// The lowest and highest coordinate the geometry reaches along `axis` through `base`.
fn extents(geometry: &Geometry, base: Point3, axis: Point3, limits: &Limits) -> Result<(f64, f64)> {
    let mut points = Vec::new();
    collect_control_points(geometry, &mut points)?;
    if points.is_empty() {
        points = sampled_points(geometry, limits)?;
    }
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for point in points {
        let distance = dot(sub(point, base), axis);
        if !distance.is_finite() {
            return Err(Error::Invalid("deformed span"));
        }
        low = low.min(distance);
        high = high.max(distance);
    }
    if !low.is_finite() || !high.is_finite() {
        return Err(Error::Invalid("deformed span"));
    }
    Ok((low, high))
}

fn collect_control_points(geometry: &Geometry, out: &mut Vec<Point3>) -> Result<()> {
    match geometry {
        Geometry::Point(point) => out.push(*point),
        Geometry::Curve(curve) => collect_curve_points(curve, out),
        Geometry::Surface(surface) => out.extend(surface.poles.iter().copied()),
        Geometry::Mesh(mesh) => out.extend(mesh.vertices.iter().copied()),
        Geometry::Cloud(cloud) => out.extend(cloud.points.iter().copied()),
        _ => {}
    }
    Ok(())
}

fn collect_curve_points(curve: &Curve, out: &mut Vec<Point3>) {
    match curve {
        Curve::Line { start, end, .. } => {
            out.push(*start);
            out.push(*end);
        }
        Curve::Circle { center, .. } => out.push(*center),
        Curve::Nurbs(nurbs) => out.extend(nurbs.poles.iter().copied()),
        Curve::Polycurve { segments, .. } => {
            for segment in segments {
                collect_curve_points(segment, out);
            }
        }
    }
}

fn sampled_points(geometry: &Geometry, limits: &Limits) -> Result<Vec<Point3>> {
    match geometry {
        Geometry::Curve(curve) => curve_ops::sample_curve(curve, LINE_SAMPLES, limits),
        Geometry::Point(point) => Ok(vec![*point]),
        _ => Ok(Vec::new()),
    }
}

/// Apply one deform to a placed object. The map runs in world space, because the base point
/// and the axis are world points the user typed, and the result is written back into the
/// object's own frame so one geometry write is enough and the placement is untouched.
pub fn apply_placed(
    deform: &Deform,
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
) -> Result<Geometry> {
    deform.validate()?;
    if let Some(reason) = unsupported(geometry) {
        return Err(Error::Unsupported(reason));
    }
    let world = crate::top11::bake_geometry(geometry, *placement, limits)?;
    let deformed = apply(deform, &world, limits)?;
    let inverse = crate::top11::inverse_affine(*placement)?;
    crate::top11::bake_geometry(&deformed, inverse, limits)
}

/// Apply one deform to a geometry. The result is validated, so a degenerate output is refused
/// instead of being committed.
pub fn apply(deform: &Deform, geometry: &Geometry, limits: &Limits) -> Result<Geometry> {
    deform.validate()?;
    // The kind is checked before the span: a flat solid also has a zero span along an axis,
    // and "deformation of a solid" is the answer a caller can act on.
    if let Some(reason) = unsupported(geometry) {
        return Err(Error::Unsupported(reason));
    }
    let span = deform.span(geometry, limits)?;
    let mapped = match geometry {
        Geometry::Point(point) => Geometry::Point(deform.map(*point, span)?),
        Geometry::Curve(curve) => Geometry::Curve(deform_curve(deform, curve, span, limits)?),
        Geometry::Surface(surface) => Geometry::Surface(deform_surface(deform, surface, span)?),
        Geometry::Mesh(mesh) => Geometry::Mesh(deform_mesh(deform, mesh, span)?),
        Geometry::Cloud(cloud) => {
            let mut deformed = cloud.clone();
            for point in &mut deformed.points {
                *point = deform.map(*point, span)?;
            }
            deformed.normals.clear();
            Geometry::Cloud(deformed)
        }
        // Refused above by kind; kept explicit so a new kind cannot slip through.
        _ => return Err(Error::Unsupported("deformation of this geometry kind")),
    };
    mapped.validate(limits)?;
    Ok(mapped)
}

/// Why a geometry kind cannot take a nonlinear deformation, or `None` when it can.
fn unsupported(geometry: &Geometry) -> Option<&'static str> {
    match geometry.kind() {
        crate::geometry::GeometryKind::Brep => Some("deformation of a solid"),
        crate::geometry::GeometryKind::Point
        | crate::geometry::GeometryKind::Curve
        | crate::geometry::GeometryKind::Surface
        | crate::geometry::GeometryKind::Mesh
        | crate::geometry::GeometryKind::Cloud => None,
        _ => Some("deformation of this geometry kind"),
    }
}

fn deform_curve(deform: &Deform, curve: &Curve, span: f64, limits: &Limits) -> Result<Curve> {
    if let Some(affine) = deform.affine() {
        return affine_curve(&affine, deform, curve, span, limits);
    }
    curve_to_polycurve(deform, curve, span, limits)
}

fn affine_curve(
    affine: &Transform,
    deform: &Deform,
    curve: &Curve,
    span: f64,
    limits: &Limits,
) -> Result<Curve> {
    match curve {
        Curve::Line { start, end, domain } => {
            let start = affine.point(*start)?;
            let end = affine.point(*end)?;
            if start == end {
                return Err(Error::Invalid("degenerate line"));
            }
            Ok(Curve::Line {
                start,
                end,
                domain: *domain,
            })
        }
        Curve::Polycurve { segments, reversed } => Ok(Curve::Polycurve {
            segments: segments
                .iter()
                .map(|segment| affine_curve(affine, deform, segment, span, limits))
                .collect::<Result<_>>()?,
            reversed: *reversed,
        }),
        Curve::Nurbs(nurbs) => {
            let mut deformed = nurbs.clone();
            for pole in &mut deformed.poles {
                *pole = affine.point(*pole)?;
            }
            Ok(Curve::Nurbs(deformed))
        }
        Curve::Circle { .. } => curve_to_polycurve(deform, curve, span, limits),
    }
}

fn deform_surface(deform: &Deform, surface: &NurbsSurface, span: f64) -> Result<NurbsSurface> {
    let mut deformed = surface.clone();
    for pole in &mut deformed.poles {
        *pole = deform.map(*pole, span)?;
    }
    Ok(deformed)
}

fn deform_mesh(deform: &Deform, mesh: &Mesh, span: f64) -> Result<Mesh> {
    let mut deformed = mesh.clone();
    for vertex in &mut deformed.vertices {
        *vertex = deform.map(*vertex, span)?;
    }
    // The stored normals belong to the undeformed surface; clearing them makes the viewer
    // recompute instead of shading with a normal that is now wrong.
    deformed.normals.clear();
    Ok(deformed)
}

/// The mapped samples of a curve as connected segments.
fn curve_to_polycurve(deform: &Deform, curve: &Curve, span: f64, limits: &Limits) -> Result<Curve> {
    sampled_curve(curve, limits, &mut |point| deform.map(point, span))
}

/// Map every sample of a curve and connect the distinct results with lines.
fn sampled_curve(
    curve: &Curve,
    limits: &Limits,
    map: &mut dyn FnMut(Point3) -> Result<Point3>,
) -> Result<Curve> {
    let samples = curve_ops::sample_curve(curve, LINE_SAMPLES, limits)?;
    let mut points = Vec::with_capacity(samples.len());
    for sample in samples {
        points.push(map(sample)?);
    }
    let mut segments = Vec::with_capacity(points.len().saturating_sub(1));
    for pair in points.windows(2) {
        if pair[0] == pair[1] {
            continue;
        }
        let length = dot(sub(pair[1], pair[0]), sub(pair[1], pair[0])).sqrt();
        segments.push(Curve::Line {
            start: pair[0],
            end: pair[1],
            domain: [0.0, length],
        });
    }
    if segments.is_empty() {
        return Err(Error::Invalid("deformation collapsed the curve"));
    }
    Ok(Curve::Polycurve {
        segments,
        reversed: false,
    })
}

/// The width of one parameter domain axis, refused when it is empty or not finite.
fn domain_span(domain: [[f64; 2]; 2], axis: usize) -> Result<f64> {
    let length = domain[axis][1] - domain[axis][0];
    if !length.is_finite() || length.abs() <= 1.0e-12 {
        return Err(Error::Invalid("flow surface domain"));
    }
    Ok(length)
}

/// The proportional parameter of `point` on `from`, expressed in `to`'s own domain. Both flow
/// families use this one rule so a shape keeps its relative placement across unequal domains.
fn carried_parameters(
    from: &NurbsSurface,
    to: &NurbsSurface,
    parameters: [f64; 2],
) -> Result<[f64; 2]> {
    let span_from_u = domain_span(from.domain, 0)?;
    let span_from_v = domain_span(from.domain, 1)?;
    let span_to_u = domain_span(to.domain, 0)?;
    let span_to_v = domain_span(to.domain, 1)?;
    let u = to.domain[0][0] + (parameters[0] - from.domain[0][0]) / span_from_u * span_to_u;
    let v = to.domain[1][0] + (parameters[1] - from.domain[1][0]) / span_from_v * span_to_v;
    Ok([u, v])
}

/// The rigid transport of one object frame from the base surface to the target surface: the
/// object's own origin keeps its `(u, v)` and the frame turns from the base normal onto the
/// target normal, so a gemstone keeps its shape and size.
pub fn rigid_flow(
    base: &NurbsSurface,
    target: &NurbsSurface,
    placement: &Transform,
    limits: &Limits,
) -> Result<Transform> {
    let origin = placement.point([0.0, 0.0, 0.0])?;
    let projection = crate::curve_surface_ops::closest_point_on_surface(base, origin, limits)?;
    let parameters = carried_parameters(base, target, projection.parameters)?;
    let landing = crate::surface_kernel::evaluate(target, parameters[0], parameters[1], limits)?;
    let from = crate::surface_kernel::normal(
        base,
        projection.parameters[0],
        projection.parameters[1],
        limits,
    )?;
    let to = crate::surface_kernel::normal(target, parameters[0], parameters[1], limits)?;
    let shift = Transform::translation(sub(landing, projection.point));
    let turn = crate::transform_session::align_rotation(from, to, projection.point)?;
    shift.compose(turn)?.compose(*placement)
}

/// Pull geometry that lies on the target surface down onto the base surface at the same
/// parameters. This is the inverse of [`flow_along_surface`], which is what the pulldown row
/// needs when the flat base is the proof sheet.
pub fn pulldown_from_target(
    base: &NurbsSurface,
    target: &NurbsSurface,
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
) -> Result<Geometry> {
    placed_map(geometry, placement, limits, &mut |point| {
        let projection = crate::curve_surface_ops::closest_point_on_surface(target, point, limits)?;
        let parameters = carried_parameters(target, base, projection.parameters)?;
        crate::surface_kernel::evaluate(base, parameters[0], parameters[1], limits)
    })
}

/* ------------------------------- surface edit ------------------------------- */

/// Move a surface's control net along the surface's own U tangent, V tangent and normal,
/// measured at each pole's Greville parameter. The frame is taken from the surface as it is
/// before the edit, so the three offsets are independent of each other and one call is one
/// step, which is what `MoveUVN` asks for.
pub fn move_uvn(
    surface: &NurbsSurface,
    normal_offset: f64,
    u_offset: f64,
    v_offset: f64,
    limits: &Limits,
) -> Result<NurbsSurface> {
    if !normal_offset.is_finite() || !u_offset.is_finite() || !v_offset.is_finite() {
        return Err(Error::Invalid("MoveUVN offset"));
    }
    if normal_offset == 0.0 && u_offset == 0.0 && v_offset == 0.0 {
        return Err(Error::Invalid("MoveUVN needs one non-zero offset"));
    }
    let [nu, nv] = surface.size;
    let mut moved = surface.clone();
    for i in 0..nu {
        for j in 0..nv {
            let u = crate::surface_kernel::greville_parameter(surface, 0, i);
            let v = crate::surface_kernel::greville_parameter(surface, 1, j);
            let frame = crate::surface_kernel::derivatives(surface, u, v, limits)?;
            let mut offset = [0.0; 3];
            if u_offset != 0.0 {
                let direction = unit(frame.du, "surface U tangent")?;
                offset = add(offset, mul(direction, u_offset));
            }
            if v_offset != 0.0 {
                let direction = unit(frame.dv, "surface V tangent")?;
                offset = add(offset, mul(direction, v_offset));
            }
            if normal_offset != 0.0 {
                let direction = unit(cross(frame.du, frame.dv), "surface normal")?;
                offset = add(offset, mul(direction, normal_offset));
            }
            let index = i * nv + j;
            moved.poles[index] = add(moved.poles[index], offset);
        }
    }
    moved.validate(limits)?;
    Ok(moved)
}

/* ------------------------------ flow and splop ------------------------------ */

/// Map every control point of a geometry with `map`. A curve is written as a polycurve
/// because these maps do not keep a straight segment straight.
pub fn mapped(
    geometry: &Geometry,
    limits: &Limits,
    map: &mut dyn FnMut(Point3) -> Result<Point3>,
) -> Result<Geometry> {
    if let Some(reason) = unsupported(geometry) {
        return Err(Error::Unsupported(reason));
    }
    let mapped = match geometry {
        Geometry::Point(point) => Geometry::Point(map(*point)?),
        Geometry::Curve(curve) => Geometry::Curve(sampled_curve(curve, limits, map)?),
        Geometry::Surface(surface) => {
            let mut deformed = surface.clone();
            for pole in &mut deformed.poles {
                *pole = map(*pole)?;
            }
            Geometry::Surface(deformed)
        }
        Geometry::Mesh(mesh) => {
            let mut deformed = mesh.clone();
            for vertex in &mut deformed.vertices {
                *vertex = map(*vertex)?;
            }
            deformed.normals.clear();
            Geometry::Mesh(deformed)
        }
        Geometry::Cloud(cloud) => {
            let mut deformed = cloud.clone();
            for point in &mut deformed.points {
                *point = map(*point)?;
            }
            deformed.normals.clear();
            Geometry::Cloud(deformed)
        }
        _ => return Err(Error::Unsupported("deformation of this geometry kind")),
    };
    mapped.validate(limits)?;
    Ok(mapped)
}

/// Run a world-space map on a placed object and write the result back into its own frame.
/// Apply a world-space point map to one object's geometry, writing the result back in the
/// object's own frame so its placement is untouched.
pub fn apply_placed_map(
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
    map: &mut dyn FnMut(Point3) -> Result<Point3>,
) -> Result<Geometry> {
    placed_map(geometry, placement, limits, map)
}

fn placed_map(
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
    map: &mut dyn FnMut(Point3) -> Result<Point3>,
) -> Result<Geometry> {
    // The kind is checked before the bake: the baker answers in the mirror vocabulary, and a
    // caller needs to hear that a solid cannot be deformed.
    if let Some(reason) = unsupported(geometry) {
        return Err(Error::Unsupported(reason));
    }
    let world = crate::top11::bake_geometry(geometry, *placement, limits)?;
    let deformed = mapped(&world, limits, map)?;
    let inverse = crate::top11::inverse_affine(*placement)?;
    crate::top11::bake_geometry(&deformed, inverse, limits)
}

/// Flow Along Curve: every point keeps its station along the source path and rides the target
/// path at the same share of the arc length, moving rigidly with the tangent there.
pub fn flow_along_curve(
    source: &Curve,
    target: &Curve,
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
) -> Result<Geometry> {
    let source_length = curve_ops::arc_length(source, limits)?;
    let target_length = curve_ops::arc_length(target, limits)?;
    if !source_length.is_finite() || source_length <= 1.0e-9 {
        return Err(Error::Invalid("flow source length"));
    }
    if !target_length.is_finite() || target_length <= 1.0e-9 {
        return Err(Error::Invalid("flow target length"));
    }
    placed_map(geometry, placement, limits, &mut |point| {
        let (fraction, station, _) =
            crate::curve_surface_ops::closest_point_on_curve(source, point, limits)?;
        let along = curve_ops::length_between(source, 0.0, fraction, limits)?;
        let share = (along / source_length).clamp(0.0, 1.0);
        let target_fraction =
            curve_ops::parameter_at_length(target, share * target_length, limits)?;
        let landing = curve_ops::point_at(target, target_fraction, limits)?;
        let from = curve_ops::tangent_at(source, fraction, limits)?;
        let to = curve_ops::tangent_at(target, target_fraction, limits)?;
        let rotation = crate::transform_session::align_rotation(from, to, station)?;
        // The rotation was built about the source station, so it stays there; the station
        // itself has to land on the target station.
        Ok(add(rotation.point(point)?, sub(landing, station)))
    })
}

/// Flow Along Surface: a point keeps its `(u, v)` on the source surface and lands on the target
/// surface at the matching parameters, turned from the source normal onto the target normal.
pub fn flow_along_surface(
    source: &NurbsSurface,
    target: &NurbsSurface,
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
) -> Result<Geometry> {
    let source_domain = source.domain;
    let target_domain = target.domain;
    let span = |domain: [[f64; 2]; 2], axis: usize| -> Result<f64> {
        let length = domain[axis][1] - domain[axis][0];
        if !length.is_finite() || length.abs() <= 1.0e-12 {
            return Err(Error::Invalid("flow surface domain"));
        }
        Ok(length)
    };
    let u_span = span(source_domain, 0)?;
    let v_span = span(source_domain, 1)?;
    let u_target = span(target_domain, 0)?;
    let v_target = span(target_domain, 1)?;
    placed_map(geometry, placement, limits, &mut |point| {
        let projection = crate::curve_surface_ops::closest_point_on_surface(source, point, limits)?;
        let u = target_domain[0][0]
            + (projection.parameters[0] - source_domain[0][0]) / u_span * u_target;
        let v = target_domain[1][0]
            + (projection.parameters[1] - source_domain[1][0]) / v_span * v_target;
        let landing = crate::surface_kernel::evaluate(target, u, v, limits)?;
        let from = crate::surface_kernel::normal(
            source,
            projection.parameters[0],
            projection.parameters[1],
            limits,
        )?;
        let to = crate::surface_kernel::normal(target, u, v, limits)?;
        let rotation = crate::transform_session::align_rotation(from, to, projection.point)?;
        // Turn the offset around the projection, then carry it to the landing point.
        Ok(add(rotation.point(point)?, sub(landing, projection.point)))
    })
}

/// Splop: wrap geometry onto a surface by moving every point to the nearest point of the
/// surface. The surface is the frame, so the objects do not need to sit near it.
pub fn splop(
    surface: &NurbsSurface,
    geometry: &Geometry,
    placement: &Transform,
    limits: &Limits,
) -> Result<Geometry> {
    placed_map(geometry, placement, limits, &mut |point| {
        Ok(crate::curve_surface_ops::closest_point_on_surface(surface, point, limits)?.point)
    })
}

#[cfg(test)]
mod tests {
    fn plane_surface() -> NurbsSurface {
        let poles = vec![
            [0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
        ];
        NurbsSurface {
            degree: [1, 1],
            size: [2, 2],
            poles,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
            domain: [[0.0, 1.0], [0.0, 1.0]],
            periodic: [false, false],
            reversed: false,
        }
    }

    fn moved(surface: &NurbsSurface, normal: f64, u: f64, v: f64) -> NurbsSurface {
        move_uvn(surface, normal, u, v, &Limits::default()).expect("MoveUVN applies")
    }

    #[test]
    fn a_rigid_flow_moves_the_frame_without_changing_its_shape() {
        let base = flat_plane();
        let tilted = tilted_plane();
        let placement = Transform::translation([5.0, 5.0, 0.0]);
        let moved = rigid_flow(&base, &tilted, &placement, &Limits::default()).expect("rigid flow");
        let origin = moved.point([0.0, 0.0, 0.0]).expect("origin");
        // The base plane's (5,5) lands on the tilted plane's (5,5), which is (5,5,5).
        assert!((origin[0] - 5.0).abs() < 1.0e-9, "{origin:?}");
        assert!((origin[1] - 5.0).abs() < 1.0e-9, "{origin:?}");
        assert!((origin[2] - 5.0).abs() < 1.0e-9, "{origin:?}");
        // Lengths are preserved: a 1 mm offset stays 1 mm.
        let offset = moved.point([1.0, 0.0, 0.0]).expect("offset");
        let length = (0..3)
            .map(|axis| (offset[axis] - origin[axis]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!((length - 1.0).abs() < 1.0e-9, "{length}");
    }

    fn flat_plane() -> NurbsSurface {
        let mut poles = Vec::new();
        for u in 0..2 {
            for v in 0..2 {
                poles.push([10.0 * u as f64, 10.0 * v as f64, 0.0]);
            }
        }
        NurbsSurface {
            degree: [1, 1],
            size: [2, 2],
            poles,
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
            domain: [[0.0, 1.0], [0.0, 1.0]],
            periodic: [false, false],
            reversed: false,
        }
    }

    fn tilted_plane() -> NurbsSurface {
        let mut surface = flat_plane();
        for pole in &mut surface.poles {
            pole[2] = pole[0];
        }
        surface
    }

    #[test]
    fn move_uvn_offsets_the_control_net_along_the_normal_u_and_v() {
        let surface = plane_surface();
        let up = moved(&surface, 1.0, 0.0, 0.0);
        assert!(up.poles.iter().all(|pole| (pole[2] - 1.0).abs() < 1.0e-9));

        let along_u = moved(&surface, 0.0, 2.0, 0.0);
        assert!(
            along_u.poles.iter().all(|pole| {
                (pole[0] - (if pole[0] > 5.0 { 12.0 } else { 2.0 })).abs() < 1.0e-9
            })
        );

        let along_v = moved(&surface, 0.0, 0.0, -3.0);
        assert!(
            along_v
                .poles
                .iter()
                .all(|pole| (pole[1] - (if pole[1] > 5.0 { 7.0 } else { -3.0 })).abs() < 1.0e-9)
        );
    }

    #[test]
    fn move_uvn_refuses_a_zero_step() {
        let surface = plane_surface();
        assert!(move_uvn(&surface, 0.0, 0.0, 0.0, &Limits::default()).is_err());
    }

    use super::*;
    use crate::geometry::Curve;

    fn line(start: Point3, end: Point3) -> Curve {
        let length = dot(sub(end, start), sub(end, start)).sqrt();
        Curve::Line {
            start,
            end,
            domain: [0.0, length],
        }
    }

    #[test]
    fn a_quarter_turn_bend_maps_the_far_end_onto_the_arc() {
        let geometry = Geometry::Curve(line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]));
        let limits = Limits::default();
        let deform = Deform::Bend {
            base: [0.0; 3],
            axis: [1.0, 0.0, 0.0],
            bend: [0.0, 1.0, 0.0],
            angle_degrees: 90.0,
        };
        let deformed = apply(&deform, &geometry, &limits).expect("bend applies");
        let end = match &deformed {
            Geometry::Curve(Curve::Polycurve { segments, .. }) => {
                match segments.last().expect("segments") {
                    Curve::Line { end, .. } => *end,
                    other => panic!("expected a line segment, found {other:?}"),
                }
            }
            other => panic!("a bend must not stay a line: {other:?}"),
        };
        // R = span / theta = 10 / (pi / 2)
        let radius = 10.0 / std::f64::consts::FRAC_PI_2;
        assert!((end[0] - radius).abs() < 1.0e-6, "{end:?}");
        assert!((end[1] - radius).abs() < 1.0e-6, "{end:?}");
        assert!(end[2].abs() < 1.0e-9, "{end:?}");
    }

    #[test]
    fn a_shear_keeps_the_line_a_line_and_moves_it_along_the_shear_direction() {
        let geometry = Geometry::Curve(line([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]));
        let limits = Limits::default();
        let deform = Deform::Shear {
            base: [0.0; 3],
            measure: [1.0, 0.0, 0.0],
            axis: [0.0, 1.0, 0.0],
            factor: 0.5,
        };
        let deformed = apply(&deform, &geometry, &limits).expect("shear applies");
        match deformed {
            Geometry::Curve(Curve::Line { start, end, .. }) => {
                assert!((start[1] - 1.0).abs() < 1.0e-9, "{start:?}");
                assert!((end[0] - 10.0).abs() < 1.0e-9, "{end:?}");
                assert!((end[1] - 6.0).abs() < 1.0e-9, "{end:?}");
            }
            other => panic!("a shear keeps a line: {other:?}"),
        }
    }

    #[test]
    fn a_twist_leaves_the_axis_alone_and_turns_the_offset_at_the_far_end() {
        let geometry = Geometry::Curve(line([0.0, 0.0, 1.0], [10.0, 0.0, 1.0]));
        let limits = Limits::default();
        let deform = Deform::Twist {
            base: [0.0; 3],
            axis: [1.0, 0.0, 0.0],
            angle_degrees: 90.0,
        };
        let deformed = apply(&deform, &geometry, &limits).expect("twist applies");
        let end = match &deformed {
            Geometry::Curve(Curve::Polycurve { segments, .. }) => match segments.last() {
                Some(Curve::Line { end, .. }) => *end,
                other => panic!("expected a line segment, found {other:?}"),
            },
            other => panic!("a twist samples the curve: {other:?}"),
        };
        // The offset (0, 0, 1) turns by +90 degrees about +X, so it lands on -Y.
        assert!((end[0] - 10.0).abs() < 1.0e-9, "{end:?}");
        assert!((end[1] + 1.0).abs() < 1.0e-6, "{end:?}");
        assert!(end[2].abs() < 1.0e-6, "{end:?}");
    }

    #[test]
    fn taper_scales_the_cross_section_over_the_span_and_a_maelstrom_also_pulls_it_in() {
        let limits = Limits::default();
        let geometry = Geometry::Curve(line([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]));
        let taper = Deform::Taper {
            base: [0.0; 3],
            axis: [1.0, 0.0, 0.0],
            factor: 3.0,
        };
        let deformed = apply(&taper, &geometry, &limits).expect("taper applies");
        let end = match &deformed {
            Geometry::Curve(Curve::Polycurve { segments, .. }) => match segments.last() {
                Some(Curve::Line { end, .. }) => *end,
                other => panic!("expected a line segment, found {other:?}"),
            },
            other => panic!("a taper samples the curve: {other:?}"),
        };
        assert!((end[1] - 3.0).abs() < 1.0e-6, "{end:?}");

        let maelstrom = Deform::Maelstrom {
            base: [0.0; 3],
            axis: [1.0, 0.0, 0.0],
            angle_degrees: 0.0,
            pull: 0.5,
        };
        let deformed = apply(&maelstrom, &geometry, &limits).expect("maelstrom applies");
        let end = match &deformed {
            Geometry::Curve(Curve::Polycurve { segments, .. }) => match segments.last() {
                Some(Curve::Line { end, .. }) => *end,
                other => panic!("expected a line segment, found {other:?}"),
            },
            other => panic!("a maelstrom samples the curve: {other:?}"),
        };
        assert!((end[1] - 0.5).abs() < 1.0e-6, "{end:?}");
    }

    #[test]
    fn a_soft_move_falls_off_to_nothing_outside_its_radius() {
        let limits = Limits::default();
        let deform = Deform::SoftMove {
            base: [0.0; 3],
            displacement: [0.0, 0.0, 4.0],
            radius: 10.0,
        };
        let near = Geometry::Point([0.0; 3]);
        let far = Geometry::Point([20.0, 0.0, 0.0]);
        match apply(&deform, &near, &limits).expect("soft move applies") {
            Geometry::Point(point) => assert!((point[2] - 4.0).abs() < 1.0e-9, "{point:?}"),
            other => panic!("expected a point: {other:?}"),
        }
        match apply(&deform, &far, &limits).expect("soft move applies") {
            Geometry::Point(point) => assert!(point[2].abs() < 1.0e-12, "{point:?}"),
            other => panic!("expected a point: {other:?}"),
        }
    }

    #[test]
    fn a_solid_is_refused_instead_of_being_faked_on_its_display_mesh() {
        let limits = Limits::default();
        let deform = Deform::Twist {
            base: [0.0; 3],
            axis: [0.0, 0.0, 1.0],
            angle_degrees: 90.0,
        };
        let geometry = Geometry::NativeBrep(crate::geometry::NativeBrep {
            display: Mesh {
                vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                faces: vec![vec![0, 1, 2]],
                normals: Vec::new(),
                colors: Vec::new(),
                corner_uvs: Vec::new(),
            },
            payload: vec![0],
            crc: 0,
        });
        let error = apply(&deform, &geometry, &limits).expect_err("a solid is refused");
        assert!(format!("{error}").contains("solid"), "{error}");
    }

    #[test]
    fn a_degenerate_bend_direction_is_refused() {
        let deform = Deform::Bend {
            base: [0.0; 3],
            axis: [1.0, 0.0, 0.0],
            bend: [2.0, 0.0, 0.0],
            angle_degrees: 45.0,
        };
        assert!(deform.validate().is_err());
    }
}
