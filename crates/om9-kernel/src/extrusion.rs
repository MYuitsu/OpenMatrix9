// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact closed curve extrusion. Rust owns admission/traversal; OCCT constructs edges,
//! planar face and prism without sampling the curve or fitting a replacement.
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
    fn om9_occt_extrude_exact(
        edges: *const EdgeInput,
        edge_count: usize,
        direction: *const f64,
        capped: c_int,
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
    build(curve, direction, 1, deflection, limits)
}

/// Extrude an exact closed curve as its side shell without planar end caps.
pub fn shell(
    curve: &Curve,
    direction: Point3,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    build(curve, direction, 0, deflection, limits)
}

/// Build an exact bounded planar face from one closed analytic or rational curve.
pub fn planar_face(curve: &Curve, deflection: f64, limits: &Limits) -> Result<Brep, CreationError> {
    build(curve, [0.0, 0.0, 1.0], 2, deflection, limits)
}

fn build(
    curve: &Curve,
    direction: Point3,
    mode: c_int,
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, CreationError> {
    let mut held = curve.validate(limits).map_err(invalid)?;
    let magnitude = direction.iter().fold(0.0_f64, |length, v| length.hypot(*v));
    if !deflection.is_finite()
        || deflection <= 0.0
        || !direction.iter().all(|v| v.is_finite())
        || !magnitude.is_finite()
        || magnitude <= 1e-12
    {
        return Err(invalid("extrusion direction/deflection"));
    }
    if !curve_ops::is_closed(curve, 1e-7, limits).map_err(invalid)? {
        return Err(invalid("extrusion needs a closed planar curve"));
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
                    mode,
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
