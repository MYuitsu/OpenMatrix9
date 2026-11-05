// SPDX-License-Identifier: LGPL-2.1-or-later
//! Read-only canonical measurements. Rust validates/budgets immutable operands;
//! OCCT owns only the synchronous geometric solver, never the document.
use crate::KernelError;
use om9_core::{
    brep::Brep,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Transform},
};
use std::ffi::{CStr, c_char, c_int};

#[repr(C)]
struct Part {
    kind: c_int,
    degree: c_int,
    periodic: c_int,
    source: *const u8,
    source_len: usize,
    points: *const Point3,
    point_count: usize,
    weights: *const f64,
    knots: *const f64,
    knot_count: usize,
    domain: [f64; 2],
    radius: f64,
    matrix: [[f64; 4]; 4],
    size: [c_int;2],
    degree_v:c_int,
    knots_v:*const f64,
    knot_count_v:usize,
    domain_v:[f64;2],
}
unsafe extern "C" {
    fn om9_occt_surface_to_face(
        source: *const Part, periodic_axes: c_int, deflection: f64, angular_deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_surface_to_face_source(
        source: *const Part, periodic_axes: c_int, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const u8, usize) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_measure_distance(
        a: *const Part,
        ac: usize,
        b: *const Part,
        bc: usize,
        value: *mut f64,
        witnesses: *mut Point3,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_measure_edge_length(
        source: *const u8,
        length: usize,
        index: c_int,
        matrix: *const f64,
        value: *mut f64,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_surface_parts(
        target: *const Part,
        tools: *const Part,
        tool_count: usize,
        pieces: *mut c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_face_isocurve(
        target: *const Part, point: *const f64, direction: c_int, tolerance: f64,
        pieces: *mut c_int, deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_surface_ray_hit(
        target: *const Part, origin: *const f64, direction: *const f64, reference: *const f64,
        tolerance: f64, output: *mut f64, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_face_ray_hit(
        target: *const Part, face_index: c_int, origin: *const f64, direction: *const f64,
        reference: *const f64, tolerance: f64, output: *mut f64, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_face_ray_hits(
        target: *const Part, face_indices: *const c_int, count: usize,
        origin: *const f64, direction: *const f64, tolerance: f64,
        output: *mut f64, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_shrink_face(
        source: *const u8, length: usize, deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_face_underlying_bounds(source: *const u8, length: usize, bounds: *mut f64,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_face_shrink_supported(source: *const u8, length: usize, supported: *mut c_int,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_face_periodicity(source: *const u8, length: usize, axes: *mut c_int,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_split_boundary_components(
        target: *const Part, tools: *const Part, tool_count: usize, pieces: *mut c_int,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_boundary_component(
        source: *const u8, length: usize, index: c_int, deflection: f64,
        max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut std::ffi::c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut std::ffi::c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
}

/// Convert a validated bounded NURBS surface into one owned OCCT face snapshot.
pub fn surface_to_face(
    surface: &om9_core::geometry::NurbsSurface, placement: Transform,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, KernelError> {
    surface_to_face_with_angle(surface, placement, deflection_mm, 0.3, limits)
}

/// Convert a NURBS surface with an explicit mesher angle in radians.
pub fn surface_to_face_with_angle(
    surface: &om9_core::geometry::NurbsSurface, placement: Transform,
    deflection_mm: f64, angular_deflection: f64, limits: &Limits,
) -> Result<Brep, KernelError> {
    if !deflection_mm.is_finite() || deflection_mm <= 0. {
        return Err(KernelError::InvalidInput("invalid surface deflection".into()));
    }
    if !angular_deflection.is_finite() || angular_deflection <= 0.0 || angular_deflection > std::f64::consts::PI {
        return Err(KernelError::InvalidInput("invalid surface angular deflection".into()));
    }
    admit(surface.validate(limits))?;
    admit(placement.validate())?;
    if surface.periodic.iter().any(|p|*p) {admit(surface.certify_repeated_periodic_basis(limits))?;}
    let mut part=surface_part(surface,placement)?;
    // Repeated control rows encode the same active surface as an unclamped
    // nonperiodic B-spline; OCCT's periodic constructor expects a different layout.
    part.periodic=0;
    let budget=crate::brep::snapshot_budget(limits);
    crate::brep::snapshot_via(deflection_mm,limits,|sink,context,message,capacity| {
        // SAFETY: validated borrowed surface arrays and part live for the synchronous call.
        unsafe {om9_occt_surface_to_face(&part,i32::from(surface.periodic[0]) | (i32::from(surface.periodic[1]) << 1),deflection_mm,angular_deflection,budget.points,budget.nodes,budget.bytes,sink,context,message,capacity)}
    })
}

/// Convert one validated NURBS face to OCCT source without creating a display mesh.
pub fn surface_to_face_source(surface: &om9_core::geometry::NurbsSurface, placement: Transform, limits: &Limits) -> Result<String, KernelError> {
    surface_to_face_source_admitted(surface, placement, limits, |_, _| Ok(()))
}
/// Admit the returned Rust source copy; OCCT conversion transients remain
/// separate from the copy callback and are not bounded by this hook.
pub fn surface_to_face_source_admitted(surface: &om9_core::geometry::NurbsSurface, placement: Transform, limits: &Limits,
    mut admit_memory: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<String, KernelError> {
    admit(surface.validate(limits))?;
    admit(placement.validate())?;
    if surface.periodic.iter().any(|p| *p) { admit(surface.certify_repeated_periodic_basis(limits))?; }
    let mut part = surface_part(surface, placement)?;
    part.periodic = 0;
    let mut context = crate::brep::AdmittedBytesContext::new(crate::brep::byte_bound(limits), &mut admit_memory);
    let mut message = [0i8; 512];
    // SAFETY: the surface arrays, callback context and output buffer outlive the synchronous call.
    let status = unsafe { om9_occt_surface_to_face_source(&part,
        i32::from(surface.periodic[0]) | (i32::from(surface.periodic[1]) << 1),
        crate::brep::byte_bound(limits), crate::brep::receive_admitted_bytes,
        (&mut context as *mut crate::brep::AdmittedBytesContext<'_>).cast(), message.as_mut_ptr(), 512) };
    if let Some(error) = context.error { return Err(KernelError::InvalidInput(error.to_string())); }
    if status != 0 { return Err(error(status, &message)); }
    if context.bytes.is_empty() { return Err(KernelError::InvalidInput("surface face source missing".into())); }
    String::from_utf8(context.bytes).map_err(|_| KernelError::InvalidInput("surface face source encoding".into()))
}
fn error(status: c_int, message: &[c_char]) -> KernelError {
    // SAFETY: the adapter NUL-terminates the fixed buffer on every error.
    let text = unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    if status == 1 {
        KernelError::InvalidInput(text)
    } else {
        KernelError::Failure(text)
    }
}
fn admit(result: om9_core::Result<impl Sized>) -> Result<(), KernelError> {
    result
        .map(|_| ())
        .map_err(|e| KernelError::InvalidInput(e.to_string()))
}
impl Part {
    fn new(transform: Transform) -> Self {
        Self {
            kind: 0,
            degree: 0,
            periodic: 0,
            source: std::ptr::null(),
            source_len: 0,
            points: std::ptr::null(),
            point_count: 0,
            weights: std::ptr::null(),
            knots: std::ptr::null(),
            knot_count: 0,
            domain: [0., 1.],
            radius: 0.,
            matrix: transform.0,
            size:[0;2],degree_v:0,knots_v:std::ptr::null(),knot_count_v:0,domain_v:[0.,1.],
        }
    }
}
fn curve_parts(
    curve: &Curve,
    transform: Transform,
    parts: &mut Vec<Part>,
    owned: &mut Vec<Vec<Point3>>,
    limits: &Limits,
) -> Result<(), KernelError> {
    if let Curve::Polycurve { segments, .. } = curve {
        for c in segments {
            curve_parts(c, transform, parts, owned, limits)?;
        }
        return Ok(());
    }
    let mut p = Part::new(transform);
    match curve {
        Curve::Line { start, end, .. } => {
            p.kind = 2;
            owned.push(vec![*start, *end]);
            p.points = owned.last().unwrap().as_ptr();
            p.point_count = 2;
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            p.kind = 3;
            owned.push(vec![*center, *x, *y]);
            p.points = owned.last().unwrap().as_ptr();
            p.point_count = 3;
            p.radius = *radius;
            p.domain = angle_domain.unwrap_or(*domain);
        }
        Curve::Nurbs(n) => {
            if n.periodic { admit(n.certify_repeated_periodic_basis(limits))?; }
            p.kind = 4;
            p.points = n.poles.as_ptr();
            p.point_count = n.poles.len();
            p.weights = n.weights.as_ptr();
            p.knots = n.knots.as_ptr();
            p.knot_count = n.knots.len();
            p.degree = n.degree as c_int;
            // Certified repeated bases have an equivalent extended nonperiodic
            // representation; OCCT's periodic constructor uses another layout.
            p.periodic = 0;
            p.domain = n.domain;
        }
        Curve::Polycurve { .. } => unreachable!(),
    }
    parts.push(p);
    Ok(())
}
fn surface_part(s: &om9_core::geometry::NurbsSurface, transform: Transform) -> Result<Part,KernelError> {
    let mut p=Part::new(transform);
    p.kind=6;p.points=s.poles.as_ptr();p.point_count=s.poles.len();p.weights=s.weights.as_ptr();
    p.degree=c_int::try_from(s.degree[0]).map_err(|_|KernelError::InvalidInput("surface U degree".into()))?;
    p.degree_v=c_int::try_from(s.degree[1]).map_err(|_|KernelError::InvalidInput("surface V degree".into()))?;
    p.periodic=i32::from(s.periodic[0])|(i32::from(s.periodic[1])<<1);
    p.size=[c_int::try_from(s.size[0]).map_err(|_|KernelError::InvalidInput("surface U size".into()))?,c_int::try_from(s.size[1]).map_err(|_|KernelError::InvalidInput("surface V size".into()))?];
    p.knots=s.knots[0].as_ptr();p.knot_count=s.knots[0].len();p.knots_v=s.knots[1].as_ptr();p.knot_count_v=s.knots[1].len();p.domain=s.domain[0];p.domain_v=s.domain[1];
    Ok(p)
}

fn parts(
    geometry: &Geometry,
    transform: Transform,
    limits: &Limits,
    owned: &mut Vec<Vec<Point3>>,
) -> Result<Vec<Part>, KernelError> {
    admit(geometry.validate(limits))?;
    admit(transform.validate())?;
    let mut out = Vec::new();
    let mut p = Part::new(transform);
    match geometry {
        Geometry::Brep(b) => {
            p.source = b.source().as_ptr();
            p.source_len = b.source().len();
            out.push(p);
        }
        Geometry::Point(point) => {
            p.kind = 1;
            p.points = point;
            p.point_count = 1;
            out.push(p);
        }
        Geometry::Curve(c) => curve_parts(c, transform, &mut out, owned, limits)?,
        Geometry::Surface(s)=>out.push(surface_part(s,transform)?),
        Geometry::Mesh(mesh) => {
            for face in &mesh.faces {
                if face.len()!=3{return Err(KernelError::InvalidInput("distance on polygon mesh needs an explicit triangulation".into()));}
                for i in 1..face.len() - 1 {
                    let mut p = Part::new(transform);
                    p.kind = 5;
                    owned.push(vec![
                        mesh.vertices[face[0] as usize],
                        mesh.vertices[face[i] as usize],
                        mesh.vertices[face[i + 1] as usize],
                    ]);
                    p.points = owned.last().unwrap().as_ptr();
                    p.point_count = 3;
                    out.push(p);
                }
            }
        }
        _ => {
            return Err(KernelError::InvalidInput(
                "object distance expects point, curve, mesh or canonical BRep".into(),
            ));
        }
    }
    if out.is_empty() || out.len() as u64 > limits.max_nodes {
        return Err(KernelError::InvalidInput(
            "object distance operand budget".into(),
        ));
    }
    Ok(out)
}
#[derive(Debug, Clone, Copy)]
pub struct Distance {
    pub millimetres: f64,
    pub points: [Point3; 2],
}

/// Split an exact surface or the boundary faces of a BRep with independently placed tools.
/// The result is an owned open BRep partition in world coordinates; no document state is changed.
pub fn split_surface_by_geometry(
    target: &Geometry,
    placement: Transform,
    cutters: &[(&Geometry, Transform)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<crate::editing::SplitResult, KernelError> {
    if !matches!(target, Geometry::Surface(_) | Geometry::Brep(_))
        || cutters.is_empty()
        || !deflection_mm.is_finite()
        || deflection_mm <= 0.0
    {
        return Err(KernelError::InvalidInput("invalid surface split inputs".into()));
    }
    if matches!(target, Geometry::Surface(surface) if surface.reversed) {
        return Err(KernelError::InvalidInput("reversed surface splitting is not supported".into()));
    }
    let mut owned = Vec::new();
    let target_parts = parts(target, placement, limits, &mut owned)?;
    let mut tools = Vec::new();
    for (geometry, transform) in cutters {
        if !matches!(geometry, Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_)) {
            return Err(KernelError::InvalidInput("invalid surface splitting tool".into()));
        }
        tools.extend(parts(geometry, *transform, limits, &mut owned)?);
        if tools.len() as u64 > limits.max_nodes {
            return Err(KernelError::InvalidInput("surface splitting tool budget".into()));
        }
    }
    let budget = crate::brep::snapshot_budget(limits);
    let mut pieces = 0;
    let brep = crate::brep::snapshot_via(deflection_mm, limits, |sink, context, message, capacity| {
        // SAFETY: geometry, owned point arrays and part buffers live until the synchronous call ends.
        unsafe {
            om9_occt_split_surface_parts(target_parts.as_ptr(), tools.as_ptr(), tools.len(),
                &mut pieces, deflection_mm, budget.points, budget.nodes, budget.bytes,
                sink, context, message, capacity)
        }
    })?;
    let pieces = usize::try_from(pieces).map_err(|_| KernelError::Failure("invalid split count".into()))?;
    let measured = crate::creation::measure(&brep, limits)
        .map_err(|e| KernelError::Failure(format!("split surface measurement: {e}")))?;
    if measured.faces != pieces {
        return Err(KernelError::Failure("split surface face count mismatch".into()));
    }
    Ok(crate::editing::SplitResult { brep, pieces })
}

/// Split one open BRep face at the U, V or both isocurves through a world point.
pub fn split_face_isocurve(
    target: &Geometry, placement: Transform, point: Point3, direction: i32,
    tolerance: Option<f64>, deflection_mm: f64, limits: &Limits,
) -> Result<crate::editing::SplitResult, KernelError> {
    if !matches!(target, Geometry::Brep(_)) || !(0..=2).contains(&direction)
        || !deflection_mm.is_finite() || deflection_mm <= 0.
        || tolerance.is_some_and(|t| !t.is_finite() || t < 0.) {
        return Err(KernelError::InvalidInput("invalid face isocurve inputs".into()));
    }
    let mut owned = Vec::new();
    let target_parts = parts(target, placement, limits, &mut owned)?;
    let budget = crate::brep::snapshot_budget(limits);
    let mut pieces = 0;
    let brep = crate::brep::snapshot_via(deflection_mm, limits, |sink, context, message, capacity| {
        // SAFETY: operand and point buffers live until the synchronous native call returns.
        unsafe { om9_occt_split_face_isocurve(target_parts.as_ptr(), point.as_ptr(), direction,
            tolerance.unwrap_or(-1.), &mut pieces, deflection_mm, budget.points, budget.nodes, budget.bytes,
            sink, context, message, capacity) }
    })?;
    let pieces = usize::try_from(pieces).map_err(|_| KernelError::Failure("invalid isocurve count".into()))?;
    let measured = crate::creation::measure(&brep, limits)
        .map_err(|e| KernelError::Failure(format!("split isocurve measurement: {e}")))?;
    if pieces < 2 || pieces as u64 > limits.max_outputs || measured.faces != pieces {
        return Err(KernelError::Failure("invalid isocurve output topology or budget".into()));
    }
    Ok(crate::editing::SplitResult { brep, pieces })
}

/// Intersect a forward world-space ray with one exact face, choosing the hit nearest a display reference.
pub fn surface_ray_hit(
    target: &Geometry, placement: Transform, origin: Point3, direction: Point3,
    reference: Point3, tolerance: f64, limits: &Limits,
) -> Result<Point3, KernelError> {
    try_surface_ray_hit(target, placement, origin, direction, reference, tolerance, limits)?
        .ok_or_else(|| KernelError::InvalidInput("viewport ray misses exact surface".into()))
}

/// Return None only for a completed ray query with no intersection. Invalid
/// input, ambiguous hits and native failures remain errors.
pub fn try_surface_ray_hit(
    target: &Geometry, placement: Transform, origin: Point3, direction: Point3,
    reference: Point3, tolerance: f64, limits: &Limits,
) -> Result<Option<Point3>, KernelError> {
    if !matches!(target, Geometry::Surface(_) | Geometry::Brep(_))
        || !origin.iter().chain(direction.iter()).chain(reference.iter()).all(|v| v.is_finite())
        || !tolerance.is_finite() || tolerance <= 0. {
        return Err(KernelError::InvalidInput("invalid surface ray input".into()));
    }
    let mut owned = Vec::new();
    let target_parts = parts(target, placement, limits, &mut owned)?;
    if target_parts.len() != 1 { return Err(KernelError::InvalidInput("surface ray needs one face".into())); }
    let mut output = [0.; 3];
    let mut message = [0 as c_char; 512];
    // SAFETY: all part buffers and fixed outputs live until the synchronous native call returns.
    let status = unsafe { om9_occt_surface_ray_hit(target_parts.as_ptr(), origin.as_ptr(), direction.as_ptr(),
        reference.as_ptr(), tolerance, output.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status == 3 { return Ok(None); }
    if status != 0 { return Err(error(status, &message)); }
    if !output.iter().all(|v| v.is_finite()) { return Err(KernelError::Failure("invalid surface ray output".into())); }
    Ok(Some(output))
}

/// Intersect a specific canonical B-rep face in world space at its first forward hit.
/// The UUID is resolved only within this validated source snapshot; callers must
/// separately verify their scene epoch, instance path, camera and source stamp.
/// `reference` is retained for adapter compatibility and validated, but does not
/// choose which intersection is visible.
pub fn brep_face_ray_hit(
    brep: &Brep, face: uuid::Uuid, placement: Transform, origin: Point3, direction: Point3,
    reference: Point3, tolerance: f64, limits: &Limits,
) -> Result<Point3, KernelError> {
    try_brep_face_ray_hit(brep, face, placement, origin, direction, reference, tolerance, limits)?
        .ok_or_else(|| KernelError::InvalidInput("viewport ray misses exact BRep face".into()))
}

/// A geometric miss is distinct from invalid input or a native failure.
pub fn try_brep_face_ray_hit(
    brep: &Brep, face: uuid::Uuid, placement: Transform, origin: Point3, direction: Point3,
    reference: Point3, tolerance: f64, limits: &Limits,
) -> Result<Option<Point3>, KernelError> {
    admit(brep.validate(limits))?;
    admit(placement.validate())?;
    if !origin.iter().chain(direction.iter()).chain(reference.iter()).all(|v| v.is_finite())
        || !tolerance.is_finite() || tolerance <= 0. {
        return Err(KernelError::InvalidInput("invalid BRep face ray input".into()));
    }
    let face_index = brep.display.faces.iter().position(|id| *id == face)
        .ok_or_else(|| KernelError::InvalidInput("BRep face ID is not in source snapshot".into()))?;
    let face_index = c_int::try_from(face_index)
        .map_err(|_| KernelError::InvalidInput("BRep face index out of range".into()))?;
    let mut part = Part::new(placement);
    part.source = brep.source().as_ptr();
    part.source_len = brep.source().len();
    let mut output = [0.; 3];
    let mut message = [0 as c_char; 512];
    // SAFETY: immutable source, part, ray and output buffers outlive the synchronous adapter.
    let status = unsafe { om9_occt_brep_face_ray_hit(&part, face_index, origin.as_ptr(), direction.as_ptr(),
        reference.as_ptr(), tolerance, output.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status == 3 { return Ok(None); }
    if status != 0 { return Err(error(status, &message)); }
    if !output.iter().all(|v| v.is_finite()) { return Err(KernelError::Failure("invalid BRep face ray output".into())); }
    Ok(Some(output))
}

/// Refine at most eight source-bound faces for one ray and placement. The
/// adapter reconstructs the source once and returns points in input order.
/// Failure on any face rejects the group; callers must check cancellation
/// between groups and revalidate their own source and view context.
pub fn brep_face_ray_hits(
    brep: &Brep, faces: &[uuid::Uuid], placement: Transform, origin: Point3,
    direction: Point3, tolerance: f64, limits: &Limits,
) -> Result<Vec<Point3>, KernelError> {
    Ok(brep_face_ray_hits_fixed(brep, faces, placement, origin, direction, tolerance, limits)?.as_slice().to_vec())
}

/// Inline storage for a bounded face-ray group; no output vector allocation.
pub struct BrepFaceRayHits {
    points: [Point3; 8],
    len: usize,
}
impl BrepFaceRayHits {
    pub fn as_slice(&self) -> &[Point3] { &self.points[..self.len] }
}

/// The fixed-storage form of [`brep_face_ray_hits`], with identical validation.
pub fn brep_face_ray_hits_fixed(
    brep: &Brep, faces: &[uuid::Uuid], placement: Transform, origin: Point3,
    direction: Point3, tolerance: f64, limits: &Limits,
) -> Result<BrepFaceRayHits, KernelError> {
    try_brep_face_ray_hits_fixed(brep, faces, placement, origin, direction, tolerance, limits)?
        .ok_or_else(|| KernelError::InvalidInput("viewport ray misses exact BRep face".into()))
}

/// Return None if any face misses; callers can then refine individually.
pub fn try_brep_face_ray_hits_fixed(
    brep: &Brep, faces: &[uuid::Uuid], placement: Transform, origin: Point3,
    direction: Point3, tolerance: f64, limits: &Limits,
) -> Result<Option<BrepFaceRayHits>, KernelError> {
    admit(brep.validate(limits))?;
    admit(placement.validate())?;
    if faces.is_empty() || faces.len() > 8
        || !origin.iter().chain(direction.iter()).all(|v| v.is_finite())
        || !tolerance.is_finite() || tolerance <= 0. {
        return Err(KernelError::InvalidInput("invalid BRep face rays input".into()));
    }
    let mut indices = [0 as c_int; 8];
    for (index, face) in faces.iter().enumerate() {
        let face_index = brep.display.faces.iter().position(|id| id == face)
            .ok_or_else(|| KernelError::InvalidInput("BRep face ID is not in source snapshot".into()))?;
        indices[index] = c_int::try_from(face_index)
            .map_err(|_| KernelError::InvalidInput("BRep face index out of range".into()))?;
    }
    let mut part = Part::new(placement);
    part.source = brep.source().as_ptr();
    part.source_len = brep.source().len();
    let mut output = [[0_f64; 3]; 8];
    let mut message = [0 as c_char; 512];
    // SAFETY: fixed arrays, source and output remain alive for this synchronous call;
    // the adapter is bounded to faces.len() <= 8 and retains no native handle.
    let status = unsafe { om9_occt_brep_face_ray_hits(&part, indices.as_ptr(), faces.len(),
        origin.as_ptr(), direction.as_ptr(), tolerance, output.as_mut_ptr().cast(),
        message.as_mut_ptr(), message.len() as c_int) };
    if status == 3 { return Ok(None); }
    if status != 0 { return Err(error(status, &message)); }
    if !output[..faces.len()].iter().flatten().all(|v| v.is_finite()) {
        return Err(KernelError::Failure("invalid BRep face rays output".into()));
    }
    Ok(Some(BrepFaceRayHits { points: output, len: faces.len() }))
}

/// Restrict the exact underlying surfaces of a nonperiodic open BRep shell.
pub fn shrink_face(face: &Brep, deflection_mm: f64, limits: &Limits) -> Result<Brep, KernelError> {
    face.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let budget = crate::brep::snapshot_budget(limits);
    crate::brep::snapshot_via(deflection_mm, limits, |sink, context, message, capacity| {
        // SAFETY: face storage and callback stay valid throughout the synchronous call.
        unsafe { om9_occt_shrink_face(face.source().as_ptr(), face.source().len(), deflection_mm,
            budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    })
}

/// Bounds of the underlying surface (not the trimmed face) of a single BRep face.
pub fn face_underlying_bounds(face: &Brep, limits: &Limits) -> Result<[f64; 4], KernelError> {
    face.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let mut bounds = [0.; 4]; let mut message = [0 as c_char; 512];
    // SAFETY: source and fixed-size output buffers remain live during the synchronous call.
    let status = unsafe { om9_occt_face_underlying_bounds(face.source().as_ptr(), face.source().len(),
        bounds.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 { return Err(error(status, &message)); }
    Ok(bounds)
}

/// Return the periodic axes of the underlying surface of a single face.
pub fn face_periodicity(face: &Brep, limits: &Limits) -> Result<[bool; 2], KernelError> {
    face.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let mut axes = [0; 2]; let mut message = [0 as c_char; 512];
    // SAFETY: input and output buffers remain live through the synchronous call.
    let status = unsafe { om9_occt_face_periodicity(face.source().as_ptr(), face.source().len(),
        axes.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 { return Err(error(status, &message)); }
    Ok([axes[0] != 0, axes[1] != 0])
}

/// Whether every face's underlying surface is eligible for the nonperiodic shrink adapter.
pub fn face_shrink_supported(face: &Brep, limits: &Limits) -> Result<bool, KernelError> {
    face.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let mut supported = 0; let mut message = [0 as c_char; 512];
    // SAFETY: the immutable input and output buffers remain live through the synchronous call.
    let status = unsafe { om9_occt_face_shrink_supported(face.source().as_ptr(), face.source().len(),
        &mut supported, message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 { return Err(error(status, &message)); }
    Ok(supported != 0)
}

/// Split a BRep boundary into connected components, retaining original face seams.
/// Result and components are in world coordinates and own their snapshots.
pub fn split_boundary_by_geometry(
    target: &Geometry, placement: Transform, cutters: &[(&Geometry, Transform)],
    deflection_mm: f64, limits: &Limits,
) -> Result<Vec<Brep>, KernelError> {
    if !matches!(target, Geometry::Brep(_)) || cutters.is_empty() || !deflection_mm.is_finite() || deflection_mm <= 0.0 {
        return Err(KernelError::InvalidInput("invalid boundary split inputs".into()));
    }
    let mut owned = Vec::new();
    let target_parts = parts(target, placement, limits, &mut owned)?;
    let mut tools = Vec::new();
    for (geometry, transform) in cutters {
        if !matches!(geometry, Geometry::Curve(_) | Geometry::Surface(_) | Geometry::Brep(_)) {
            return Err(KernelError::InvalidInput("invalid boundary splitting tool".into()));
        }
        tools.extend(parts(geometry, *transform, limits, &mut owned)?);
        if tools.len() as u64 > limits.max_nodes {
            return Err(KernelError::InvalidInput("boundary splitting tool budget".into()));
        }
    }
    let budget = crate::brep::snapshot_budget(limits);
    let mut count = 0;
    let partition = crate::brep::snapshot_via(deflection_mm, limits, |sink, context, message, capacity| {
        // SAFETY: part arrays and owned data live for the synchronous call.
        unsafe { om9_occt_split_boundary_components(target_parts.as_ptr(), tools.as_ptr(), tools.len(),
            &mut count, deflection_mm, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    })?;
    if count < 2 || count as u64 > limits.max_objects || count as u64 > limits.max_outputs {
        return Err(KernelError::Failure("invalid boundary component count".into()));
    }
    let mut result = Vec::new();
    result.try_reserve_exact(count as usize).map_err(|_| KernelError::Failure("boundary component allocation".into()))?;
    let mut held = partition.validate(limits).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    held.add(om9_core::budget::Usage::array::<Brep>(result.capacity())
        .map_err(|e| KernelError::InvalidInput(e.to_string()))?, limits)
        .map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    for index in 0..count {
        let remaining = limits.remaining_after(held).map_err(|e| KernelError::InvalidInput(e.to_string()))?;
        let budget = crate::brep::snapshot_budget(&remaining);
        let piece = crate::brep::snapshot_via(deflection_mm, &remaining, |sink, context, message, capacity| {
            // SAFETY: immutable partition and callback remain live throughout the synchronous call.
            unsafe { om9_occt_split_boundary_component(partition.source().as_ptr(), partition.source().len(), index,
                deflection_mm, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
        })?;
        held.add(piece.validate(&remaining).map_err(|e| KernelError::InvalidInput(e.to_string()))?, limits)
            .map_err(|e| KernelError::InvalidInput(e.to_string()))?;
        result.push(piece);
    }
    Ok(result)
}

pub fn distance(
    a: &Geometry,
    at: Transform,
    b: &Geometry,
    bt: Transform,
    limits: &Limits,
) -> Result<Distance, KernelError> {
    let mut owned = Vec::new();
    let first = parts(a, at, limits, &mut owned)?;
    let second = parts(b, bt, limits, &mut owned)?;
    let mut value = 0.;
    let mut points = [[0.; 3]; 2];
    let mut message = [0; 512];
    // SAFETY: borrowed geometry and all owned point buffers remain live; no pointer is retained.
    let status = unsafe {
        om9_occt_measure_distance(
            first.as_ptr(),
            first.len(),
            second.as_ptr(),
            second.len(),
            &mut value,
            points.as_mut_ptr(),
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(error(status, &message));
    }
    Ok(Distance {
        millimetres: value,
        points,
    })
}
pub fn edge_length(
    brep: &Brep,
    index: Option<usize>,
    transform: Transform,
    limits: &Limits,
) -> Result<f64, KernelError> {
    admit(brep.validate(limits))?;
    admit(transform.validate())?;
    let index = match index {
        Some(i) if i < brep.display.edges.len() => {
            c_int::try_from(i).map_err(|_| KernelError::InvalidInput("edge index".into()))?
        }
        Some(_) => return Err(KernelError::InvalidInput("edge index".into())),
        None => -1,
    };
    let mut value = 0.;
    let mut message = [0; 512];
    // SAFETY: all inputs/outputs live through this synchronous scalar operation.
    let status = unsafe {
        om9_occt_measure_edge_length(
            brep.source().as_ptr(),
            brep.source().len(),
            index,
            transform.0.as_ptr().cast(),
            &mut value,
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(error(status, &message));
    }
    Ok(value)
}
