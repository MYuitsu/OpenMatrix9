// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact wire extrusions to a distance, boundary face or point. Rust admits and traverses the
//! profiles. OCCT constructs the analytic or rational edges and the resulting faces.
pub mod boundary;

use crate::{
    brep::{snapshot_budget, snapshot_via},
    creation::CreationError,
};
use om9_core::{
    brep::Brep,
    budget::Limits,
    curve_ops,
    geometry::{Curve, Point3},
};
use std::ffi::{c_char, c_int, c_void};

#[repr(C)]
struct EdgeInput {
    kind: c_int,
    degree: c_int,
    reversed: c_int,
    params: [f64; 12],
    poles: *const f64,
    pole_count: usize,
    weights: *const f64,
    weight_count: usize,
    knots: *const f64,
    knot_count: usize,
    domain: [f64; 2],
}
unsafe extern "C" {
    fn om9_occt_clip_draft(
        edges: *const EdgeInput,
        count: usize,
        body: *const c_char,
        body_len: usize,
        face: *const c_char,
        face_len: usize,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_extrude_exact(
        edges: *const EdgeInput,
        edge_count: usize,
        direction: *const f64,
        capped: c_int,
        angle: f64,
        corner_handle: extern "C" fn(f64, f64) -> f64,
        boundary: *const c_char,
        boundary_len: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}
fn invalid(error: impl std::fmt::Display) -> CreationError {
    CreationError::Invalid(error.to_string())
}

fn append(curve: &Curve, reverse: bool, edges: &mut Vec<EdgeInput>) -> Result<(), CreationError> {
    if let Curve::Polycurve { segments, reversed } = curve {
        let reverse = reverse ^ reversed;
        for index in 0..segments.len() {
            let index = if reverse {
                segments.len() - 1 - index
            } else {
                index
            };
            append(&segments[index], reverse, edges)?;
        }
        return Ok(());
    }
    if edges.len() >= 4096 {
        return Err(CreationError::Budget("extrusion edge count".into()));
    }
    edges
        .try_reserve(1)
        .map_err(|_| CreationError::Budget("extrusion edges".into()))?;
    let mut edge = EdgeInput {
        kind: 0,
        degree: 0,
        reversed: i32::from(reverse),
        params: [0.0; 12],
        poles: std::ptr::null(),
        pole_count: 0,
        weights: std::ptr::null(),
        weight_count: 0,
        knots: std::ptr::null(),
        knot_count: 0,
        domain: [0.0, 1.0],
    };
    match curve {
        Curve::Line { start, end, .. } => {
            edge.params[..3].copy_from_slice(start);
            edge.params[3..6].copy_from_slice(end);
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            edge.kind = 1;
            edge.params[..3].copy_from_slice(center);
            edge.params[3..6].copy_from_slice(x);
            edge.params[6..9].copy_from_slice(y);
            edge.params[9] = *radius;
            edge.domain = angle_domain.unwrap_or(*domain);
        }
        Curve::Nurbs(n) => {
            if (n.periodic || n.seam.is_some())
                && n.domain != [n.knots[n.degree], n.knots[n.poles.len()]]
            {
                return Err(invalid(
                    "trimmed periodic NURBS extrusion requires a full closed basis domain",
                ));
            }
            edge.kind = 2;
            edge.degree = c_int::try_from(n.degree).map_err(invalid)?;
            edge.reversed = i32::from(reverse ^ n.reversed);
            // Point3 is [f64;3]; its contiguous array representation is shared read-only.
            edge.poles = n.poles.as_ptr().cast();
            edge.pole_count = n.poles.len();
            edge.weights = n.weights.as_ptr();
            edge.weight_count = n.weights.len();
            edge.knots = n.knots.as_ptr();
            edge.knot_count = n.knots.len();
            edge.domain = n.domain;
        }
        Curve::Polycurve { .. } => unreachable!(),
    }
    edges.push(edge);
    Ok(())
}

/// Extrude a closed planar analytic/rational curve as one solid. Signed directions
/// are retained. A failed edge, open/nonplanar boundary or invalid body returns an
/// error; the caller publishes only a successful complete snapshot.
pub fn solid(
    curve: &Curve,
    direction: Point3,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    build(
        curve,
        direction,
        Mode::SolidPrism,
        0.,
        None,
        deflection,
        limits,
    )
}

/// Build in the source curve's local frame while accepting a world-space offset.
pub fn placed_solid(
    curve: &Curve,
    placement: om9_core::geometry::Transform,
    direction: Point3,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let inverse = om9_core::top11::inverse_affine(placement).map_err(invalid)?;
    let local = std::array::from_fn(|i| (0..3).map(|j| inverse.0[i][j] * direction[j]).sum());
    solid(curve, local, deflection, limits)
}

/// Extrude an exact closed curve as its side shell without planar end caps.
pub fn shell(
    curve: &Curve,
    direction: Point3,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    build(
        curve,
        direction,
        Mode::OpenPrism,
        0.,
        None,
        deflection,
        limits,
    )
}

/// Build an exact bounded planar face from one closed analytic or rational curve.
pub fn planar_face(curve: &Curve, deflection: f64, limits: &Limits) -> Result<Brep, CreationError> {
    build(
        curve,
        [0.0, 0.0, 1.0],
        Mode::Face,
        0.,
        None,
        deflection,
        limits,
    )
}

/// Extrude the exact source wire into side faces; open and spatial wires are valid.
pub fn side_wall(
    curve: &Curve,
    offset: Point3,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    build(curve, offset, Mode::SideWall, 0., None, deflection, limits)
}

/// Split a closed prism by a bounded face in the same coordinate system.
/// The offset must carry the far cap beyond the complete cutting face.
pub fn solid_to_face(
    curve: &Curve,
    offset: Point3,
    face: &om9_core::brep::CanonicalBrep,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    if face.topology().faces.len() != 1 {
        return Err(invalid("Select one boundary face"));
    }
    let available = limits
        .remaining_after(face.validate_source(limits).map_err(invalid)?)
        .map_err(invalid)?;
    build(
        curve,
        offset,
        Mode::SolidPrism,
        0.,
        Some(face.source()),
        deflection,
        &available,
    )
}

/// Join each exact profile edge to one vertex. The point is in the profile frame.
pub fn to_point(
    curve: &Curve,
    apex: Point3,
    solid: bool,
    boundary: Option<&om9_core::brep::CanonicalBrep>,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let mut available = *limits;
    if let Some(face) = boundary {
        if face.topology().faces.len() != 1 {
            return Err(invalid("Select one boundary face"));
        }
        available = limits
            .remaining_after(face.validate_source(limits).map_err(invalid)?)
            .map_err(invalid)?;
    }
    build(
        curve,
        apex,
        if solid {
            Mode::CappedApex
        } else {
            Mode::OpenApex
        },
        0.,
        boundary.map(|face| face.source()),
        deflection,
        &available,
    )
}

pub use om9_core::curve_ops::extrude_lines::DraftCorners;

// End handles of the cubic top edge for a smooth outside corner. Both follow the
// adjoining straight tangents. Rhino 5's right and unequal-angle fixtures fix the scale.
extern "C" fn smooth_draft_handle(chord: f64, tangent_cosine: f64) -> f64 {
    om9_core::curve_ops::extrude_lines::smooth_draft_handle(chord, tangent_cosine)
}

/// Extrusion with signed height in profile coordinates. Positive draft is to the
/// left of the directed profile as seen from the positive axis, for either sign of height.
pub fn tapered(
    curve: &Curve,
    axis: Point3,
    height: f64,
    angle: f64,
    corners: DraftCorners,
    solid: bool,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let span = axis[0].hypot(axis[1]).hypot(axis[2]);
    if !span.is_finite()
        || span == 0.
        || !height.is_finite()
        || height.abs() < 1e-12
        || !angle.is_finite()
        || angle == 0.
        || angle.abs() >= std::f64::consts::FRAC_PI_2
    {
        return Err(invalid("invalid taper direction, height or draft angle"));
    }
    let unit_axis = axis.map(|x| x / span);
    let offset = unit_axis.map(|x| x * height);
    fn single_line(curve: &Curve) -> bool {
        match curve {
            Curve::Line { .. } => true,
            Curve::Nurbs(n) => n.degree == 1 && n.poles.len() == 2,
            Curve::Polycurve { segments, .. } => segments.len() == 1 && single_line(&segments[0]),
            Curve::Circle { .. } => false,
        }
    }
    if single_line(curve) {
        return tapered_line(curve, unit_axis, height, angle, deflection, limits);
    }
    let world = om9_core::geometry::Transform::identity();
    let closed = curve_ops::is_closed(curve, 1e-7, limits).map_err(invalid)?;
    let plane = if closed {
        curve_ops::extrude_lines::profile_plane([(curve, world)], limits).map_err(invalid)?
    } else {
        None
    };
    let capped = solid && plane.is_some();
    if curve_ops::extrude_lines::is_piecewise_linear(curve)
        && plane.is_some_and(|n| curve_ops::extrude_lines::is_inclined_draft(n, unit_axis))
    {
        let vertices =
            curve_ops::linear_path::placed_vertices(curve, world, 1e-7, limits, &mut || Ok(()))
                .map_err(invalid)?;
        let count = vertices.len() - 1 + usize::from(capped);
        if count > 4097 || count as u64 > limits.max_nodes {
            return Err(CreationError::Budget("inclined taper faces".into()));
        }
        let mut held =
            om9_core::budget::Usage::array::<Point3>(vertices.capacity()).map_err(invalid)?;
        held.add(
            om9_core::budget::Usage::array::<Brep>(count).map_err(invalid)?,
            limits,
        )
        .map_err(invalid)?;
        let mut faces = Vec::with_capacity(count);
        // Each side has its own far rail on an inclined polygon, even at Sharp corners.
        for pair in vertices.windows(2) {
            let line = Curve::Line {
                start: pair[0],
                end: pair[1],
                domain: [0., 1.],
            };
            let room = limits.remaining_after(held).map_err(invalid)?;
            let face = tapered_line(&line, unit_axis, height, angle, deflection, &room)?;
            held.add(face.storage_usage(&room).map_err(invalid)?, limits)
                .map_err(invalid)?;
            faces.push(face);
        }
        if capped {
            let room = limits.remaining_after(held).map_err(invalid)?;
            let base = planar_face(curve, deflection, &room)?;
            held.add(base.storage_usage(&room).map_err(invalid)?, limits)
                .map_err(invalid)?;
            faces.push(base);
        }
        let room = limits.remaining_after(held).map_err(invalid)?;
        return crate::editing::sew_many(&faces, 1e-7, deflection, &room).map_err(invalid);
    }
    let mode = match corners {
        DraftCorners::Sharp => Mode::SharpDraft,
        DraftCorners::Round => Mode::RoundDraft,
        DraftCorners::Smooth => Mode::SmoothDraft,
    };
    let walls = build(
        curve,
        offset,
        mode,
        angle * height.signum(),
        None,
        deflection,
        limits,
    )?;
    if !capped {
        return Ok(walls);
    }
    let available = limits
        .remaining_after(walls.storage_usage(limits).map_err(invalid)?)
        .map_err(invalid)?;
    crate::editing::cap_planar(&walls, 1e-7, deflection, &available).map_err(invalid)
}

fn tapered_line(
    curve: &Curve,
    unit_axis: Point3,
    height: f64,
    angle: f64,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let offset = unit_axis.map(|v| v * height);
    let [start, end] = curve_ops::curve_endpoints(curve, limits).map_err(invalid)?;
    let line = Curve::Line {
        start,
        end,
        domain: [0., 1.],
    };
    // Rhino measures the draft angle orthogonally to the picked direction.
    // A straight rail is therefore an ordinary prism with one sideways shift.
    let shifted = om9_core::curve_modify::offset_curve(
        &line,
        -height.abs() * angle.tan(),
        unit_axis,
        4,
        limits,
    )
    .map_err(invalid)?;
    let [p, _] = curve_ops::curve_endpoints(&shifted, limits).map_err(invalid)?;
    side_wall(
        &line,
        std::array::from_fn(|i| offset[i] + (p[i] - start[i])),
        deflection,
        limits,
    )
}

fn clip_draft(
    curve: &Curve,
    body: &Brep,
    face: &om9_core::brep::CanonicalBrep,
    tolerance: f64,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    use om9_core::budget::Usage;
    let mut usage = curve.validate(limits).map_err(invalid)?;
    usage
        .add(body.storage_usage(limits).map_err(invalid)?, limits)
        .map_err(invalid)?;
    usage
        .add(face.storage_usage(limits).map_err(invalid)?, limits)
        .map_err(invalid)?;
    let mut edges = Vec::new();
    append(curve, false, &mut edges)?;
    usage
        .add(
            Usage::array::<EdgeInput>(edges.capacity()).map_err(invalid)?,
            limits,
        )
        .map_err(invalid)?;
    let room = limits.remaining_after(usage).map_err(invalid)?;
    let budget = snapshot_budget(&room);
    snapshot_via(deflection, &room, |sink, ctx, message, capacity| {
        // SAFETY: the validated profile and both complete shape buffers outlive this synchronous call.
        unsafe {
            om9_occt_clip_draft(
                edges.as_ptr(),
                edges.len(),
                body.source().as_ptr().cast(),
                body.source().len(),
                face.source().as_ptr().cast(),
                face.source().len(),
                tolerance,
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                ctx,
                message,
                capacity,
            )
        }
    })
    .map_err(CreationError::from)
}

#[repr(i32)]
#[derive(Clone, Copy)]
enum Mode {
    OpenPrism,
    SolidPrism,
    Face,
    SideWall,
    OpenApex,
    CappedApex,
    SharpDraft,
    RoundDraft,
    SmoothDraft,
}

// OCCT can exhaust its stack when a ruled face collapses onto its base line.
// For a rational curve the entire control hull must be collinear to reject it here.
fn collapsed_apex_panel(curve: &Curve, apex: Point3) -> bool {
    fn collinear(points: &[Point3], apex: Point3) -> bool {
        let origin = points[0];
        let delta = |p: Point3| -> Point3 { std::array::from_fn(|i| p[i] - origin[i]) };
        let length = |d: Point3| d[0].hypot(d[1]).hypot(d[2]);
        let direction = points
            .iter()
            .copied()
            .map(delta)
            .max_by(|a, b| length(*a).total_cmp(&length(*b)))
            .unwrap();
        let span = length(direction);
        if span == 0. {
            return true;
        }
        let [x, y, z] = direction.map(|v| v / span);
        let on_line = |p| {
            let [a, b, c] = delta(p);
            length([y * c - z * b, z * a - x * c, x * b - y * a]) <= 1e-7
        };
        on_line(apex) && points.iter().copied().all(on_line)
    }
    match curve {
        Curve::Line { start, end, .. } => collinear(&[*start, *end], apex),
        Curve::Nurbs(n) => collinear(&n.poles, apex),
        Curve::Polycurve { segments, .. } => segments.iter().any(|c| collapsed_apex_panel(c, apex)),
        Curve::Circle { .. } => false,
    }
}

fn build(
    curve: &Curve,
    direction: Point3,
    mode: Mode,
    angle: f64,
    boundary: Option<&str>,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let mut held = curve.validate(limits).map_err(invalid)?;
    let magnitude = direction.iter().fold(0.0_f64, |length, v| length.hypot(*v));
    if !deflection.is_finite()
        || deflection <= 0.0
        || !direction.iter().all(|v| v.is_finite())
        || !magnitude.is_finite()
        || (!matches!(mode, Mode::OpenApex | Mode::CappedApex) && magnitude <= 1e-12)
    {
        return Err(invalid("extrusion direction/deflection"));
    }
    if !matches!(
        mode,
        Mode::SideWall | Mode::OpenApex | Mode::SharpDraft | Mode::RoundDraft | Mode::SmoothDraft
    ) && !curve_ops::is_closed(curve, 1e-7, limits).map_err(invalid)?
    {
        return Err(invalid("extrusion needs a closed planar curve"));
    }
    if matches!(mode, Mode::OpenApex | Mode::CappedApex) && collapsed_apex_panel(curve, direction) {
        return Err(invalid(
            "Pick an apex off the line of each straight profile edge",
        ));
    }
    let mut edges = Vec::new();
    append(curve, false, &mut edges)?;
    held.add(
        om9_core::budget::Usage::array::<EdgeInput>(edges.capacity()).map_err(invalid)?,
        limits,
    )
    .map_err(invalid)?;
    let output_limits = limits.remaining_after(held).map_err(invalid)?;
    let budget = snapshot_budget(&output_limits);
    snapshot_via(
        deflection,
        &output_limits,
        |sink, context, message, capacity| {
            // SAFETY: all pointers borrow validated Rust arrays through this synchronous
            // operation. The native adapter retains none; snapshot_via owns the callback.
            unsafe {
                om9_occt_extrude_exact(
                    edges.as_ptr(),
                    edges.len(),
                    direction.as_ptr(),
                    mode as c_int,
                    angle,
                    smooth_draft_handle,
                    boundary.map_or(std::ptr::null(), |s| s.as_ptr().cast()),
                    boundary.map_or(0, str::len),
                    deflection,
                    budget.points,
                    budget.nodes,
                    budget.bytes,
                    sink,
                    context,
                    message,
                    capacity,
                )
            }
        },
    )
    .map_err(CreationError::from)
}
