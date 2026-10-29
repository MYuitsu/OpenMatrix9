// SPDX-License-Identifier: LGPL-2.1-or-later
//! Geometry editing operations (M03 `OM9-L2-01-10`): B-Rep booleans, curve/face split, planar cap,
//! fillet/chamfer, offset/thicken, face extract/unjoin/rejoin and geometry health.
//!
//! Every operation takes owned [`Brep`] inputs (canonical OCCT source), validates them in safe
//! Rust, performs one synchronous OCCT construction through `editing_bridge.cpp`, and returns one
//! owned [`Brep`]. Failure never mutates an input: the inputs are immutable values and the adapter
//! builds a new shape.
//!
//! Edge and face selection is by index into the canonical OCCT map order, which is also the order
//! of [`Brep::display`]'s `edges`/`faces`. [`edge_endpoints`] is a selection helper derived from the
//! display snapshot; acceptance measurements always come from [`creation::measure`].
use crate::{
    brep::{snapshot_budget, snapshot_via, snapshot_via_optional},
    creation::{CreationError, Measures, measure},
};
use om9_core::{brep::Brep, budget::Limits, geometry::Point3};
use std::ffi::{c_char, c_int, c_void};

/// The typed refusal contract is shared with the creation owner.
pub type EditingError = CreationError;

type NativeSink = unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int;

unsafe extern "C" {
    fn om9_occt_coplanar_faces_plane(source: *const u8, length: usize, plane: *mut f64,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_boss_cylinder_carrier(source: *const u8, length: usize, axis_data: *mut f64, extension: f64,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: NativeSink, context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_trim_surface_plane(source: *const u8, length: usize, plane: *const f64, remove_point: *const f64,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_boolean(
        op: c_int,
        source_a: *const u8,
        length_a: usize,
        source_b: *const u8,
        length_b: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_face(
        source: *const u8,
        length: usize,
        line: *const f64,
        line_count: usize,
        pieces: *mut c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_solid(
        source_a: *const u8,
        length_a: usize,
        source_b: *const u8,
        length_b: usize,
        pieces: *mut c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_create_solid_regions(
        sources: *const *const u8,
        lengths: *const usize,
        count: usize,
        regions: *mut c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_split_solid_piece(
        source: *const u8,
        length: usize,
        index: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_cap(
        source: *const u8,
        length: usize,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_fillet(
        source: *const u8,
        length: usize,
        edges: *const c_int,
        edge_count: usize,
        radius: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_fillet_radii(
        source: *const u8,
        length: usize,
        edges: *const c_int,
        radii: *const f64,
        edge_count: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_fillet_end_radii(
        source: *const u8,
        length: usize,
        edges: *const c_int,
        start_radii: *const f64,
        end_radii: *const f64,
        edge_count: usize,
        control_edges: *const c_int,
        control_fractions: *const f64,
        control_radii: *const f64,
        control_count: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_oriented_edge_endpoints(
        source: *const u8,
        length: usize,
        edge: c_int,
        points: *mut f64,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_edge_is_line(source: *const u8, length: usize, edge: c_int,
        is_line: *mut c_int, message: *mut std::ffi::c_char, capacity: c_int) -> c_int;
    fn om9_occt_fillet_shared_endpoints(source: *const u8, length: usize, anchor: c_int, side: c_int,
        candidates: *const c_int, count: usize, masks: *mut u8, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_circular_edge(source: *const u8, length: usize, edge: c_int,
        output: *mut f64, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_fillet_project_edge(source: *const u8, length: usize, edge: c_int,
        point: *const f64, output: *mut f64, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_chamfer(
        source: *const u8,
        length: usize,
        edges: *const c_int,
        edge_count: usize,
        distance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_thicken(
        source: *const u8,
        length: usize,
        thickness: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_offset_shape(
        source: *const u8,
        length: usize,
        distance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_extract_face(
        source: *const u8,
        length: usize,
        face_index: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_remove_face(
        source: *const u8,
        length: usize,
        face_index: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_remove_faces(
        source: *const u8,
        length: usize,
        face_indices: *const c_int,
        count: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sew(
        source_a: *const u8,
        length_a: usize,
        source_b: *const u8,
        length_b: usize,
        reverse_b: c_int,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
}

fn checked_deflection(deflection_mm: f64) -> Result<f64, EditingError> {
    if !deflection_mm.is_finite() || deflection_mm <= 0.0 {
        return Err(EditingError::Invalid(
            "deflection must be finite and positive".into(),
        ));
    }
    Ok(deflection_mm)
}

fn checked_brep(brep: &Brep, limits: &Limits) -> Result<Measures, EditingError> {
    measure(brep, limits).map_err(|error| match error {
        CreationError::Invalid(message) => CreationError::Invalid(message),
        other => other,
    })
}

fn positive(value: f64, what: &str) -> Result<(), EditingError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(EditingError::Invalid(format!(
            "{what} must be finite and positive"
        )));
    }
    Ok(())
}

fn edge_table(edges: &[usize], available: usize) -> Result<Vec<c_int>, EditingError> {
    if edges.is_empty() {
        return Err(EditingError::Invalid("no edge selected".into()));
    }
    let mut table = Vec::with_capacity(edges.len());
    for index in edges {
        if *index >= available {
            return Err(EditingError::Invalid(format!(
                "edge index {index} is outside the {available} published edges"
            )));
        }
        table.push(i32::try_from(*index).map_err(|_| EditingError::Budget("edge index".into()))?);
    }
    Ok(table)
}

/* ------------------------- booleans ------------------------- */

/// The three B-Rep boolean operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

impl BooleanOp {
    fn native(self) -> c_int {
        match self {
            Self::Union => 0,
            Self::Difference => 1,
            Self::Intersection => 2,
        }
    }
}

/// A boolean outcome. An intersection of disjoint solids is a legitimate empty result, not an
/// error and not a fabricated zero-face body.
///
/// The body variant is deliberately inline: a boolean's normal outcome is the body and boxing it
/// would add one allocation per successful operation; `Empty` is the rare variant.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanOutcome {
    Body(Brep),
    Empty,
}

impl BooleanOutcome {
    pub fn body(&self) -> Option<&Brep> {
        match self {
            Self::Body(brep) => Some(brep),
            Self::Empty => None,
        }
    }
}

/// Union, difference or intersection of two closed solids.
pub fn boolean(
    a: &Brep,
    b: &Brep,
    op: BooleanOp,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let measures_a = checked_brep(a, limits)?;
    let measures_b = checked_brep(b, limits)?;
    if measures_a.solids == 0 || measures_b.solids == 0 {
        return Err(EditingError::Invalid(
            "boolean requires two closed solids".into(),
        ));
    }
    let budget = snapshot_budget(limits);
    let brep = snapshot_via_optional(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: both sources, the callback context and every output live through this call.
        unsafe {
            om9_occt_boolean(
                op.native(),
                a.source().as_ptr(),
                a.source().len(),
                b.source().as_ptr(),
                b.source().len(),
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
    })
    .map_err(EditingError::from)?;
    Ok(match brep {
        Some(brep) => BooleanOutcome::Body(brep),
        None => BooleanOutcome::Empty,
    })
}

/// Cut a surface-only B-Rep by a solid tool, preserving exact face geometry.
/// A disjoint tool returns the unchanged surface; complete removal returns Empty.
/// The caller owns classification, publication and any subsequent joining.
pub fn trim_surface_by_solid(
    surface: &Brep,
    tool: &Brep,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let source = checked_brep(surface, limits)?;
    let cutter = checked_brep(tool, limits)?;
    if source.faces == 0 || source.solids != 0 {
        return Err(EditingError::Invalid("surface trim requires surface-only input".into()));
    }
    if cutter.solids == 0 || !cutter.closed {
        return Err(EditingError::Invalid("surface trim requires a closed solid tool".into()));
    }
    let budget = snapshot_budget(limits);
    let result = snapshot_via_optional(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: validated sources and callback context remain alive during the synchronous call.
        unsafe { om9_occt_boolean(BooleanOp::Difference.native(), surface.source().as_ptr(), surface.source().len(),
            tool.source().as_ptr(), tool.source().len(), deflection, budget.points, budget.nodes,
            budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)?;
    Ok(match result { Some(brep) => BooleanOutcome::Body(brep), None => BooleanOutcome::Empty })
}

/// Intersect an open surface/shell with a closed solid without changing either input.
/// Empty intersections are reported explicitly; classification belongs to the caller.
pub fn intersect_surface_with_solid(
    surface: &Brep, solid: &Brep, deflection_mm: f64, limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let source = checked_brep(surface, limits)?;
    let cutter = checked_brep(solid, limits)?;
    if source.faces == 0 || source.solids != 0 { return Err(EditingError::Invalid("surface intersection requires surface-only input".into())); }
    if cutter.solids == 0 || !cutter.closed { return Err(EditingError::Invalid("surface intersection requires a closed solid".into())); }
    let budget = snapshot_budget(limits);
    let result = snapshot_via_optional(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: validated sources and callback context remain alive during the synchronous call.
        unsafe { om9_occt_boolean(BooleanOp::Intersection.native(), surface.source().as_ptr(), surface.source().len(),
            solid.source().as_ptr(), solid.source().len(), deflection, budget.points, budget.nodes,
            budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)?;
    Ok(match result { Some(brep) => BooleanOutcome::Body(brep), None => BooleanOutcome::Empty })
}

/// Construct a finite solid carrier of a single cylindrical face and return its
/// axis origin, unit direction and radius; neither native handles nor source ownership escape.
pub fn boss_cylinder_carrier(boundary: &Brep, deflection_mm: f64, limits: &Limits) -> Result<(Brep, [f64;7]), EditingError> {
    boss_cylinder_carrier_extended(boundary, 0.0, deflection_mm, limits)
}

fn boss_cylinder_carrier_extended(boundary: &Brep, extension: f64, deflection_mm: f64, limits: &Limits) -> Result<(Brep, [f64;7]), EditingError> {
    if !extension.is_finite() || extension < 0.0 { return Err(EditingError::Invalid("Boss cylinder extension".into())); }
    let deflection = checked_deflection(deflection_mm)?;
    let source = checked_brep(boundary, limits)?;
    if source.faces != 1 || source.solids != 0 { return Err(EditingError::Invalid("Boss cylinder requires one surface face".into())); }
    let mut axis = [0.0;7];
    let budget = snapshot_budget(limits);
    let carrier = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source, axis output and callback context live throughout this synchronous call.
        unsafe { om9_occt_boss_cylinder_carrier(boundary.source().as_ptr(),boundary.source().len(),axis.as_mut_ptr(),extension,
            deflection,budget.points,budget.nodes,budget.bytes,sink,context,message,capacity) }
    }).map_err(EditingError::from)?;
    if !axis.iter().all(|v| v.is_finite()) || axis[6] <= 0.0 { return Err(EditingError::Kernel("invalid Boss cylinder axis".into())); }
    Ok((carrier, axis))
}

/// Construct a polygon Boss on one complete cylindrical face, choosing the
/// polygon's positive normal. The entire polygon must lie inside the carrier.
pub fn boss_cylindrical_polygon(profile: &crate::creation::Profile, boundary: &Brep, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    boss_cylindrical_polygons(&[profile], boundary, deflection_mm, limits)
}

/// Build disjoint polygon Boss walls on one cylindrical face as one result.
pub fn boss_cylindrical_polygons(profiles: &[&crate::creation::Profile], boundary: &Brep, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    boss_cylindrical_impl(profiles.iter().map(|p| CylindricalSection::Polygon(p)),profiles.len(),boundary,deflection_mm,limits)
}

/// Build disjoint exact polygon or curve Boss walls on one cylindrical face.
pub fn boss_cylindrical_sections(profiles: &[BossSection], boundary: &Brep, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    boss_cylindrical_impl(profiles.iter().map(|s| match s {
        BossSection::Polygon(p) => CylindricalSection::Polygon(p),
        BossSection::Circle(c) => CylindricalSection::Circle(c),
        BossSection::Curve(c) => CylindricalSection::Curve(c),
    }),profiles.len(),boundary,deflection_mm,limits)
}

enum CylindricalSection<'a> { Polygon(&'a crate::creation::Profile), Circle(&'a om9_core::geometry::Curve), Curve(&'a om9_core::geometry::Curve) }

// Refine clamped NURBS at existing knots, then use the exact rational Bezier
// intervals as distinct edges. Unsupported domains retain the original curve.
fn boss_bezier_chain(n: &om9_core::geometry::NurbsCurve, limits: &Limits) -> Result<Option<om9_core::geometry::Curve>, EditingError> {
    use om9_core::geometry::{Curve,NurbsCurve};
    let mut planned=n.validate(limits).map_err(|e| EditingError::Invalid(e.to_string()))?;
    let p=n.degree;
    if n.periodic || n.seam.is_some() || n.domain != [n.knots[p],n.knots[n.poles.len()]] { return Ok(None); }
    if n.knots[..=p].iter().any(|k| *k != n.domain[0]) || n.knots[n.poles.len()..].iter().any(|k| *k != n.domain[1]) { return Ok(None); }
    let mut missing=0usize; let mut spans=1usize; let mut scan=p+1;
    while scan<n.poles.len() {
        let knot=n.knots[scan]; let mut end=scan+1;
        while end<n.poles.len() && n.knots[end]==knot { end+=1; }
        if knot<=n.domain[0] || knot>=n.domain[1] || end-scan>p { return Ok(None); }
        missing=missing.checked_add(p-(end-scan)).ok_or_else(||EditingError::Budget("Boss knot refinement".into()))?;
        spans=spans.checked_add(1).ok_or_else(||EditingError::Budget("Boss Bezier segment count".into()))?;
        scan=end;
    }
    if spans as u64>limits.max_outputs || spans as u64>limits.max_nodes {return Err(EditingError::Budget("Boss Bezier segment count".into()));}
    if missing>0 {
        // Knot insertion uses simultaneous homogeneous input/output and knot
        // buffers. Admit their maximum sizes before invoking the shared routine.
        let max_poles=n.poles.len().checked_add(missing).ok_or_else(||EditingError::Budget("Boss refined poles".into()))?;
        let max_knots=n.knots.len().checked_add(missing).ok_or_else(||EditingError::Budget("Boss refined knots".into()))?;
        for _ in 0..2 {
            planned.add(om9_core::budget::Usage::array::<[f64;4]>(max_poles).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
            planned.add(om9_core::budget::Usage::array::<f64>(max_knots).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
            planned.add(om9_core::budget::Usage {points:max_poles as u64,..om9_core::budget::Usage::array::<Point3>(max_poles).map_err(|e|EditingError::Budget(e.to_string()))?},limits).map_err(|e|EditingError::Budget(e.to_string()))?;
            planned.add(om9_core::budget::Usage::array::<f64>(max_poles).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
        }
        let mut refined=n.clone(); let mut index=p+1;
        while index<n.poles.len() {
            let knot=n.knots[index]; let mut end=index+1;
            while end<n.poles.len() && n.knots[end]==knot { end+=1; }
            let add=p-(end-index);
            if add>0 {
                refined=om9_core::curve_build::insert_knot(&refined,knot,add,limits).map_err(|e|EditingError::Invalid(e.to_string()))?;
                refined.reversed=n.reversed;
            }
            index=end;
        }
        let remaining=limits.remaining_after(n.validate(limits).map_err(|e|EditingError::Budget(e.to_string()))?).map_err(|e|EditingError::Budget(e.to_string()))?;
        return boss_bezier_chain(&refined,&remaining);
    }
    planned.add(om9_core::budget::Usage::array::<f64>(n.knots.len()).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    let mut breaks=Vec::new();
    breaks.try_reserve_exact(n.knots.len()).map_err(|_| EditingError::Budget("Boss Bezier knots".into()))?;
    breaks.push(n.domain[0]);
    let mut i=p+1;
    while i<n.poles.len() {
        let knot=n.knots[i]; let mut end=i+1;
        while end<n.poles.len() && n.knots[end]==knot { end+=1; }
        if knot<=n.domain[0] || knot>=n.domain[1] || end-i!=p { return Ok(None); }
        breaks.push(knot); i=end;
    }
    breaks.push(n.domain[1]);
    let count=breaks.len()-1;
    if count<=1 { return Ok(None); }
    if count as u64>limits.max_outputs || count as u64>limits.max_nodes || count.checked_mul(p).and_then(|v|v.checked_add(1))!=Some(n.poles.len()) {
        return Err(EditingError::Budget("Boss Bezier segment count".into()));
    }
    let copied=count.checked_mul(p+1).ok_or_else(||EditingError::Budget("Boss Bezier pole count".into()))?;
    let knot_total=copied.checked_mul(2).ok_or_else(||EditingError::Budget("Boss Bezier knot count".into()))?;
    planned.add(om9_core::budget::Usage {points:copied as u64,..om9_core::budget::Usage::array::<Point3>(copied).map_err(|e| EditingError::Budget(e.to_string()))?},limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    planned.add(om9_core::budget::Usage::array::<f64>(copied).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    planned.add(om9_core::budget::Usage::array::<f64>(knot_total).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    planned.add(om9_core::budget::Usage::array::<Curve>(count).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    let mut segments=Vec::new(); segments.try_reserve_exact(count).map_err(|_| EditingError::Budget("Boss Bezier segments".into()))?;
    for j in 0..count {
        let first=j*p; let last=first+p+1;
        let mut poles=Vec::new(); poles.try_reserve_exact(p+1).map_err(|_| EditingError::Budget("Boss Bezier poles".into()))?; poles.extend_from_slice(&n.poles[first..last]);
        let mut weights=Vec::new(); weights.try_reserve_exact(p+1).map_err(|_| EditingError::Budget("Boss Bezier weights".into()))?; weights.extend_from_slice(&n.weights[first..last]);
        let knot_count=(p+1).checked_mul(2).ok_or_else(||EditingError::Budget("Boss Bezier knots".into()))?;
        let mut knots=Vec::new(); knots.try_reserve_exact(knot_count).map_err(|_| EditingError::Budget("Boss Bezier knots".into()))?;
        knots.resize(p+1,breaks[j]); knots.resize(knot_count,breaks[j+1]);
        segments.push(Curve::Nurbs(NurbsCurve {degree:p,poles,weights,knots,domain:[breaks[j],breaks[j+1]],periodic:false,reversed:false,seam:None}));
    }
    let curve=Curve::Polycurve {segments,reversed:n.reversed};
    curve.validate(limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    Ok(Some(curve))
}

// Refine eligible NURBS inside a polycurve as well as a standalone curve.
// Preserve segment order and each reversal; unsupported segments are copied
// unchanged and still pass through the ordinary exact extrusion validation.
fn boss_refine_curve(curve: &om9_core::geometry::Curve, limits: &Limits, depth: usize) -> Result<Option<om9_core::geometry::Curve>, EditingError> {
    use om9_core::geometry::Curve;
    fn has_refinable(curve: &Curve) -> bool {
        match curve {
            Curve::Nurbs(n) => !n.periodic && n.seam.is_none() && n.domain==[n.knots[n.degree],n.knots[n.poles.len()]] && n.knots.iter().any(|k| *k>n.domain[0] && *k<n.domain[1]),
            Curve::Polycurve {segments,..} => segments.iter().any(has_refinable),
            _ => false,
        }
    }
    if depth > limits.max_depth { return Err(EditingError::Budget("Boss curve depth".into())); }
    match curve {
        Curve::Nurbs(n) => boss_bezier_chain(n,limits),
        Curve::Polycurve {segments,reversed} => {
            let mut held=curve.validate(limits).map_err(|e|EditingError::Invalid(e.to_string()))?;
            if !segments.iter().any(has_refinable) {return Ok(None);}
            held.add(om9_core::budget::Usage::array::<Curve>(segments.len()).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
            let mut out=Vec::new(); out.try_reserve_exact(segments.len()).map_err(|_|EditingError::Budget("Boss polycurve segments".into()))?;
            let mut changed=false;
            for segment in segments {
                let remaining=limits.remaining_after(held).map_err(|e|EditingError::Budget(e.to_string()))?;
                let replacement=boss_refine_curve(segment,&remaining,depth+1)?;
                let item=match replacement {
                  Some(Curve::Polycurve {segments:mut pieces,reversed:reverse})=>{
                    changed=true;
                    held.add(om9_core::budget::Usage::array::<Curve>(pieces.len()).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
                    out.try_reserve_exact(pieces.len()).map_err(|_|EditingError::Budget("Boss refined polycurve segments".into()))?;
                    if reverse {pieces.reverse();}
                    for mut piece in pieces {
                        if reverse {
                            piece=match piece {
                                Curve::Nurbs(mut n)=>{n.reversed=!n.reversed; Curve::Nurbs(n)},
                                Curve::Line {start,end,domain}=>Curve::Line {start:end,end:start,domain},
                                other=>om9_core::curve_ops::reverse_curve(&other,&remaining).map_err(|e|EditingError::Invalid(e.to_string()))?,
                            };
                        }
                        held.add(piece.validate(limits).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
                        out.push(piece);
                    }
                    continue;
                  }
                    Some(refined)=>{changed=true; refined},
                    None=>{ segment.validate(&remaining).map_err(|e|EditingError::Budget(e.to_string()))?; segment.clone() },
                };
                held.add(item.validate(limits).map_err(|e|EditingError::Budget(e.to_string()))?,limits).map_err(|e|EditingError::Budget(e.to_string()))?;
                out.push(item);
            }
            if changed {Ok(Some(Curve::Polycurve {segments:out,reversed:*reversed}))} else {Ok(None)}
        }
        _ => Ok(None),
    }
}

fn boss_cylindrical_impl<'a>(profiles: impl Iterator<Item=CylindricalSection<'a>>, count: usize, boundary: &Brep, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    if count == 0 || count as u64 > limits.max_outputs {
        return Err(EditingError::Invalid("Boss needs bounded profile selection".into()));
    }
    let original = checked_brep(boundary, limits)?;
    if original.faces != 1 || original.solids != 0 || original.naked_edges != 2 {
        return Err(EditingError::Invalid("Boss cylinder requires one complete cylindrical face".into()));
    }
    let (carrier, cylinder) = boss_cylinder_carrier(boundary, deflection_mm, limits)?;
    let axis = [cylinder[3], cylinder[4], cylinder[5]];
    let origin = [cylinder[0], cylinder[1], cylinder[2]];
    let radius = cylinder[6];
    let original_measure = measure(boundary, limits)?;
    let original_area = original_measure.area_mm2;
    let mut axis_lo=f64::INFINITY; let mut axis_hi=f64::NEG_INFINITY;
    for corner in 0..8 {
        let t=(0..3).map(|i| (original_measure.bounds_mm[i+if corner & (1<<i)!=0 {3} else {0}]-origin[i])*axis[i]).sum::<f64>();
        axis_lo=axis_lo.min(t); axis_hi=axis_hi.max(t);
    }
    let mut cut = boundary.clone();
    let mut walls = Vec::new();
    walls.try_reserve_exact(count).map_err(|_| EditingError::Budget("Boss cylinder walls".into()))?;
    let mut held = carrier.validate(limits).map_err(|e| EditingError::Budget(e.to_string()))?;
    held.add(om9_core::budget::Usage::array::<Brep>(walls.capacity()).map_err(|e| EditingError::Budget(e.to_string()))?, limits)
        .map_err(|e| EditingError::Budget(e.to_string()))?;
    let mut edge_count = 0usize;
    for profile in profiles {
    let segmented=match &profile {
        CylindricalSection::Curve(curve) => {
            let remaining=limits.remaining_after(held).map_err(|e| EditingError::Budget(e.to_string()))?;
            boss_refine_curve(curve,&remaining,0)?
        },
        _ => None,
    };
    if let Some(curve)=&segmented { held.add(curve.validate(limits).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?; }
    let profile=match segmented.as_ref() {Some(curve)=>CylindricalSection::Curve(curve),None=>profile};
    let (n, edges, bounds) = match &profile {
        CylindricalSection::Polygon(p) => (p.normal(),p.points().len(),None),
        CylindricalSection::Circle(curve) => {
            let om9_core::geometry::Curve::Circle {x,y,..} = curve else { return Err(EditingError::Invalid("Boss circle section requires a circle".into())); };
            let face=crate::extrusion::planar_face(curve,deflection_mm,limits)?;
            let m=measure(&face,limits)?;
            ([x[1]*y[2]-x[2]*y[1],x[2]*y[0]-x[0]*y[2],x[0]*y[1]-x[1]*y[0]],m.naked_edges,Some(m.bounds_mm))
        }
        CylindricalSection::Curve(curve) => {
            let face=crate::extrusion::planar_face(curve,deflection_mm,limits)?;
            let plane=planar_faces_plane(&face,false,limits)?;
            let m=measure(&face,limits)?;
            ([plane[3],plane[4],plane[5]],m.naked_edges,Some(m.bounds_mm))
        }
    };
    let axial = (0..3).map(|i| axis[i]*n[i]).sum::<f64>();
    let radial_direction=(0..3).map(|i| (n[i]-axial*axis[i]).powi(2)).sum::<f64>().sqrt();
    if !axial.is_finite() || !radial_direction.is_finite() || radial_direction <= 1e-9 { return Err(EditingError::Invalid("Boss direction must cross cylinder axis".into())); }
    let mut profile_lo=f64::INFINITY; let mut profile_hi=f64::NEG_INFINITY;
    let mut check_point = |point: Point3| -> Result<(),EditingError> {
        let delta: Point3 = std::array::from_fn(|i| point[i]-origin[i]);
        let along = (0..3).map(|i| delta[i]*axis[i]).sum::<f64>();
        let radial = (0..3).map(|i| (delta[i]-along*axis[i]).powi(2)).sum::<f64>().sqrt();
        if !radial.is_finite() || radial >= radius-1e-6 { return Err(EditingError::Invalid("Boss profile must lie inside cylinder".into())); }
        profile_lo=profile_lo.min(along); profile_hi=profile_hi.max(along);
        Ok(())
    };
    if let CylindricalSection::Polygon(p)=&profile { for point in p.points() { check_point(*point)?; } }
    else if let Some(bounds)=bounds { for corner in 0..8 { check_point(std::array::from_fn(|axis| bounds[axis+if corner & (1<<axis)!=0 {3} else {0}]))?; } }
    let length=2.1*radius/radial_direction;
    let shift=axial*length;
    let extension=(axis_lo-profile_lo.min(profile_lo+shift)).max(profile_hi.max(profile_hi+shift)-axis_hi).max(0.0);
    if !length.is_finite() || !extension.is_finite() { return Err(EditingError::Invalid("Boss cylinder extent overflow".into())); }
    let extended = if extension > 0.0 {
        let (body,_) = boss_cylinder_carrier_extended(boundary,extension,deflection_mm,limits)?;
        let mut transient=held;
        transient.add(body.validate(limits).map_err(|e| EditingError::Budget(e.to_string()))?,limits).map_err(|e| EditingError::Budget(e.to_string()))?;
        Some(body)
    } else { None };
    let direction = n.map(|v| v * length);
    let (tool,wall) = match profile {
        CylindricalSection::Polygon(p) => (crate::creation::extrude(p,direction,true,deflection_mm,limits)?,crate::creation::extrude(p,direction,false,deflection_mm,limits)?),
        CylindricalSection::Circle(c) | CylindricalSection::Curve(c) => (crate::extrusion::solid(c,direction,deflection_mm,limits)?,crate::extrusion::shell(c,direction,deflection_mm,limits)?),
    };
    let baseline = trim_surface_by_solid(boundary, &tool, deflection_mm, limits)?;
    let baseline = baseline.body().ok_or_else(|| EditingError::Invalid("Boss removes the entire cylinder boundary".into()))?;
    let expected = original_area - measure(baseline, limits)?.area_mm2;
    let before = measure(&cut, limits)?.area_mm2;
    let next = trim_surface_by_solid(&cut, &tool, deflection_mm, limits)?;
    let next = next.body().ok_or_else(|| EditingError::Invalid("Boss removes the entire cylinder boundary".into()))?;
    let removed = before - measure(next, limits)?.area_mm2;
    if !expected.is_finite() || expected <= 1e-8 || !removed.is_finite() || (removed-expected).abs() > 1e-7 * expected.max(1.0) {
        return Err(EditingError::Invalid("Boss profile misses or overlaps another cylinder profile".into()));
    }
    let clipped = intersect_surface_with_solid(&wall, extended.as_ref().unwrap_or(&carrier), deflection_mm, limits)?;
    let clipped = clipped.body().ok_or_else(|| EditingError::Invalid("Boss cylinder walls are empty".into()))?;
    held.add(clipped.validate(limits).map_err(|e| EditingError::Budget(e.to_string()))?, limits)
        .map_err(|e| EditingError::Budget(e.to_string()))?;
    edge_count = edge_count.checked_add(edges).ok_or_else(|| EditingError::Budget("Boss edge count".into()))?;
    cut = next.clone(); walls.push(clipped.clone());
    }
    let mut result = cut;
    for wall in &walls { result = sew(&result,wall,1e-7,deflection_mm,limits)?; }
    let m = measure(&result, limits)?;
    let faces = original.faces.checked_add(edge_count).ok_or_else(|| EditingError::Budget("Boss face count".into()))?;
    let naked = original.naked_edges.checked_add(edge_count).ok_or_else(|| EditingError::Budget("Boss naked edge count".into()))?;
    if m.solids != 0 || m.shells != 1 || m.faces != faces || m.naked_edges != naked {
        return Err(EditingError::Invalid(format!("Boss cylinder walls did not join the complete boundary (faces {} expected {faces}, shells {}, solids {}, naked edges {} expected {naked})",m.faces,m.shells,m.solids,m.naked_edges)));
    }
    Ok(result)
}

/// Trim a surface by an infinite plane, removing the side containing `remove_point`.
/// The kernel returns a new snapshot; callers must classify the resulting pieces.
pub fn trim_surface_by_plane(
    surface: &Brep, origin: Point3, normal: Point3, remove_point: Point3,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let source = checked_brep(surface, limits)?;
    if source.faces == 0 || source.solids != 0 { return Err(EditingError::Invalid("plane trim requires surface-only input".into())); }
    if !origin.iter().chain(normal.iter()).chain(remove_point.iter()).all(|v| v.is_finite()) {
        return Err(EditingError::Invalid("plane trim requires finite inputs".into()));
    }
    let magnitude = normal.iter().map(|v| v*v).sum::<f64>().sqrt();
    if !magnitude.is_finite() || magnitude <= 1e-12 { return Err(EditingError::Invalid("plane trim normal".into())); }
    let normal = normal.map(|v| v / magnitude);
    let side = (0..3).map(|i| (remove_point[i] - origin[i])*normal[i]).sum::<f64>();
    if !side.is_finite() || side.abs() <= 1e-7 { return Err(EditingError::Invalid("plane trim point on plane".into())); }
    let plane = [origin[0], origin[1], origin[2], normal[0], normal[1], normal[2]];
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: validated immutable source, arrays and callback context live through the synchronous call.
        unsafe { om9_occt_trim_surface_plane(surface.source().as_ptr(), surface.source().len(), plane.as_ptr(), remove_point.as_ptr(),
            deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)
}

/// Construct the open joined Boss shell for two simple parallel polygonal profiles.
/// The boundary is reconstructed from its profile; no document objects are mutated.
/// Returns an error when the projected boss crosses or misses the boundary.
pub fn boss_parallel_profiles(
    profile: &crate::creation::Profile,
    boundary: &crate::creation::Profile,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let face = crate::creation::profile_to_brep(boundary, deflection_mm, limits)?;
    boss_parallel_planar_face(profile, &face, deflection_mm, limits)
}

/// Construct a Boss shell against a bounded planar B-Rep boundary, with a
/// simple polygonal profile. Inputs are immutable.
pub fn boss_parallel_planar_face(
    profile: &crate::creation::Profile,
    boundary: &Brep,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    boss_parallel_planar_face_many(std::slice::from_ref(profile), boundary, deflection_mm, limits)
}

/// Development foundation for a straight, one-sided in-curve-plane Rib.
/// It builds a mitered strip from an open polyline and joins walls to a planar
/// boundary using the existing Boss adapter. This is not the full Rib command.
pub fn rib_straight_planar_boundary(
    points: &[om9_core::geometry::Point3], normal: om9_core::geometry::Point3,
    distance: f64, boundary: &Brep, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    rib_straight_planar_boundary_many(&[(points, normal)], distance, boundary, deflection_mm, limits)
}

/// Development-only multi-curve version. All strips must be disjoint and
/// individually valid; construction fails without changing any input.
pub fn rib_straight_planar_boundary_many(
    curves: &[(&[om9_core::geometry::Point3], om9_core::geometry::Point3)],
    distance: f64, boundary: &Brep, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    if curves.is_empty() || curves.len() as u64 > limits.max_outputs {
        return Err(EditingError::Invalid("Rib needs bounded curve selection".into()));
    }
    let mut profiles = Vec::new();
    profiles.try_reserve_exact(curves.len()).map_err(|_| EditingError::Budget("Rib profiles".into()))?;
    for &(points, normal) in curves {
    let (mut perimeter,_) = om9_core::slab::polyline_slab(points,normal,distance,1.,false,limits)
        .map_err(|e|EditingError::Invalid(e.to_string()))?;
    let mut signed_area=0.;
    for i in 0..perimeter.len() {
        let a=perimeter[i];let b=perimeter[(i+1)%perimeter.len()];
        signed_area+=(a[1]*b[2]-a[2]*b[1])*normal[0]+(a[2]*b[0]-a[0]*b[2])*normal[1]+(a[0]*b[1]-a[1]*b[0])*normal[2];
    }
    if signed_area < 0. {
        perimeter.reverse();
    }
    profiles.push(crate::creation::Profile::new(perimeter)?);
    }
    boss_parallel_planar_face_many(&profiles,boundary,deflection_mm,limits)
}

/// Construct one joined shell from disjoint polygon profiles and a planar boundary.
/// Any invalid profile fails the entire operation without modifying the inputs.
pub fn boss_parallel_planar_face_many(
    profiles: &[crate::creation::Profile],
    boundary: &Brep,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    if profiles.is_empty() || profiles.len() as u64 > limits.max_outputs {
        return Err(EditingError::Invalid("Boss needs bounded profile selection".into()));
    }
    let mut sections = Vec::new();
    sections.try_reserve_exact(profiles.len()).map_err(|_| EditingError::Budget("Boss sections".into()))?;
    sections.extend(profiles.iter().cloned().map(BossSection::Polygon));
    boss_parallel_planar_sections(&sections, boundary, deflection_mm, limits)
}

/// Supported exact closed sections for the planar Boss foundation.
#[derive(Debug, Clone)]
pub enum BossSection {
    Polygon(crate::creation::Profile),
    Circle(om9_core::geometry::Curve),
    Curve(om9_core::geometry::Curve),
}

// Project the union of line endpoints and positive-weight NURBS control hulls.
// Unsupported segment families return None so the caller keeps the BRep box.
fn boss_nurbs_line_hull(curve: &om9_core::geometry::Curve, q: Point3, b: Point3, dot: f64, limits: &Limits) -> Result<Option<(f64,f64)>, EditingError> {
    fn visit(curve: &om9_core::geometry::Curve, depth: usize, limits: &Limits, q: Point3, b: Point3, dot: f64, range: &mut [f64;3]) -> Result<bool, EditingError> {
        if depth > limits.max_depth { return Err(EditingError::Budget("Boss profile depth".into())); }
        let mut point = |p: Point3| -> Result<(), EditingError> {
            let t = (0..3).map(|i| (q[i]-p[i])*b[i]).sum::<f64>() / dot;
            let scale = (0..3).map(|i| (q[i].abs()+p[i].abs())*b[i].abs()).sum::<f64>() / dot.abs();
            if !t.is_finite() || !scale.is_finite() { return Err(EditingError::Invalid("Boss profile extent overflow".into())); }
            range[0]=range[0].min(t); range[1]=range[1].max(t); range[2]=range[2].max(scale); Ok(())
        };
        match curve {
            om9_core::geometry::Curve::Line { start,end,.. } => { point(*start)?; point(*end)?; },
            om9_core::geometry::Curve::Nurbs(n) => for p in &n.poles { point(*p)?; },
            om9_core::geometry::Curve::Polycurve { segments,.. } => {
                for segment in segments { if !visit(segment, depth+1, limits, q, b, dot, range)? { return Ok(false); } }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    let mut range=[f64::INFINITY,f64::NEG_INFINITY,0.0];
    if !visit(curve,0,limits,q,b,dot,&mut range)? { return Ok(None); }
    if !range[0].is_finite() || !range[1].is_finite() { return Err(EditingError::Invalid("Boss empty profile hull".into())); }
    let pad=1e-10+64.0*f64::EPSILON*range[2];
    Ok(Some((range[0]-pad,range[1]+pad)))
}

fn planar_faces_plane(boundary: &Brep, allow_multiple: bool, limits: &Limits) -> Result<[f64; 6], EditingError> {
    let original = checked_brep(boundary, limits)?;
    if original.faces == 0 || (!allow_multiple && original.faces != 1) || original.solids != 0 || (allow_multiple && original.faces > 1 && original.shells != 1) {
        return Err(EditingError::Invalid("Boss boundary requires one connected surface or shell".into()));
    }
    let mut plane = [0.0_f64; 6]; let mut message = [0 as c_char; 512];
    // SAFETY: immutable source and fixed output buffers remain live during the synchronous call.
    let status = unsafe { om9_occt_coplanar_faces_plane(boundary.source().as_ptr(), boundary.source().len(),
        plane.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        // SAFETY: the message buffer is zero-initialized and remains NUL terminated.
        let text = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned();
        return Err(EditingError::Invalid(text));
    }
    if !plane.iter().all(|v| v.is_finite()) { return Err(EditingError::Kernel("invalid Boss plane".into())); }
    Ok(plane)
}

pub fn boss_parallel_planar_sections(
    sections: &[BossSection],
    boundary: &Brep,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    if sections.is_empty() || sections.len() as u64 > limits.max_outputs {
        return Err(EditingError::Invalid("Boss needs bounded profile selection".into()));
    }
    let original = checked_brep(boundary, limits)?;
    let plane = planar_faces_plane(boundary, true, limits)?;
    let b = [plane[3], plane[4], plane[5]];
    let q = [plane[0], plane[1], plane[2]];
    let make = |e: crate::creation::CreationError| EditingError::Invalid(e.to_string());
    let mut cut = boundary.clone();
    let mut walls = Vec::new();
    walls.try_reserve_exact(sections.len()).map_err(|_| EditingError::Budget("Boss walls".into()))?;
    let mut total_edges = 0usize;
    for section in sections {
    let (n, p, area, edges) = match section {
        BossSection::Polygon(profile) => (profile.normal(), profile.centroid(), profile.area_mm2(), profile.points().len()),
        BossSection::Circle(curve) => {
            curve.validate(limits).map_err(|e| EditingError::Invalid(e.to_string()))?;
            let om9_core::geometry::Curve::Circle { center, x, y, radius, domain, angle_domain } = curve else {
                return Err(EditingError::Invalid("Boss circle section requires a circle".into()));
            };
            let angles = angle_domain.unwrap_or(*domain);
            if ((angles[1]-angles[0])-std::f64::consts::TAU).abs() > 1e-9 {
                return Err(EditingError::Invalid("Boss circle must be closed".into()));
            }
            let normal = [x[1]*y[2]-x[2]*y[1], x[2]*y[0]-x[0]*y[2], x[0]*y[1]-x[1]*y[0]];
            (normal, *center, std::f64::consts::PI * radius * radius, 1)
        }
        BossSection::Curve(curve) => {
            let face = crate::extrusion::planar_face(curve, deflection_mm, limits).map_err(make)?;
            let plane = planar_faces_plane(&face, false, limits)?;
            let m = measure(&face, limits)?;
            ([plane[3], plane[4], plane[5]], [plane[0], plane[1], plane[2]], m.area_mm2, m.naked_edges)
        }
    };
    let cross = [n[1]*b[2]-n[2]*b[1], n[2]*b[0]-n[0]*b[2], n[0]*b[1]-n[1]*b[0]];
    let tilted = cross.iter().map(|v| v*v).sum::<f64>().sqrt() > 1e-9;
    let dot = (0..3).map(|i| n[i]*b[i]).sum::<f64>();
    if dot.abs() < 1e-9 { return Err(EditingError::Invalid("Boss direction is parallel to boundary".into())); }
    let distance = (0..3).map(|i| (q[i]-p[i])*b[i]).sum::<f64>() / dot;
    if !distance.is_finite() || (!tilted && distance.abs() <= 1e-6) {
        return Err(EditingError::Invalid("Boss boundary must be separated from profile".into()));
    }
    let length = if tilted {
        let mut minimum = f64::INFINITY;
        let mut maximum = f64::NEG_INFINITY;
        let mut include = |point: Point3| -> Result<(), EditingError> {
            let t = (0..3).map(|i| (q[i]-point[i])*b[i]).sum::<f64>() / dot;
            if !t.is_finite() { return Err(EditingError::Invalid("Boss profile extent".into())); }
            minimum = minimum.min(t); maximum = maximum.max(t); Ok(())
        };
        match section {
            BossSection::Polygon(profile) => for point in profile.points() { include(*point)?; },
            BossSection::Circle(om9_core::geometry::Curve::Circle { center, x, y, radius, .. }) => {
                let bx = (0..3).map(|i| b[i]*x[i]).sum::<f64>();
                let by = (0..3).map(|i| b[i]*y[i]).sum::<f64>();
                let projection = bx.hypot(by);
                if projection <= 1e-15 { include(*center)?; } else {
                    let offset: Point3 = std::array::from_fn(|i| radius*(bx*x[i]+by*y[i])/projection);
                    include(std::array::from_fn(|i| center[i]+offset[i]))?;
                    include(std::array::from_fn(|i| center[i]-offset[i]))?;
                }
            }
            BossSection::Curve(curve) => {
                // A conservative exact-BRep bounding box may reject a valid near-plane profile.
                let face = crate::extrusion::planar_face(curve, deflection_mm, limits).map_err(make)?;
                let bounds = measure(&face, limits)?.bounds_mm;
                for corner in 0..8 {
                    include(std::array::from_fn(|axis| bounds[axis + if corner & (1<<axis) != 0 { 3 } else { 0 }]))?;
                }
            }
            BossSection::Circle(_) => return Err(EditingError::Invalid("Boss circle section requires a circle".into())),
        }
        if let BossSection::Curve(curve) = section {
            if let Some((hull_min,hull_max)) = boss_nurbs_line_hull(curve,q,b,dot,limits)? {
                minimum = minimum.max(hull_min);
                maximum = maximum.min(hull_max);
            }
        }
        if minimum > maximum { return Err(EditingError::Invalid("Boss inconsistent profile bounds".into())); }
        if minimum > 1e-6 { 2.0 * maximum }
        else if maximum < -1e-6 { 2.0 * minimum }
        else { return Err(EditingError::Invalid("Boss boundary crosses or approaches profile plane".into())); }
    } else { distance };
    let direction = n.map(|v| v*length);
    let tool = match section {
        BossSection::Polygon(profile) => crate::creation::extrude(profile, direction.map(|v| if tilted { v } else { 2.0*v }), true, deflection_mm, limits),
        BossSection::Circle(curve) => crate::extrusion::solid(curve, direction.map(|v| if tilted { v } else { 2.0*v }), deflection_mm, limits),
        BossSection::Curve(curve) => crate::extrusion::solid(curve, direction.map(|v| if tilted { v } else { 2.0*v }), deflection_mm, limits),
    }.map_err(make)?;
    let before = measure(&cut, limits)?;
    let outcome = trim_surface_by_solid(&cut, &tool, deflection_mm, limits)?;
    let next = outcome.body().ok_or_else(|| EditingError::Invalid("Boss removes the entire boundary".into()))?;
    let trimmed = measure(next, limits)?;
    let removed = before.area_mm2 - trimmed.area_mm2;
    let projected_area = area / dot.abs();
    if (removed - projected_area).abs() > 1e-7 * projected_area.max(1.0) {
        return Err(EditingError::Invalid("Boss profile must lie inside the finite boundary".into()));
    }
    let wall = match section {
        BossSection::Polygon(profile) => crate::creation::extrude(profile, direction, false, deflection_mm, limits),
        BossSection::Circle(curve) => crate::extrusion::shell(curve, direction, deflection_mm, limits),
        BossSection::Curve(curve) => crate::extrusion::shell(curve, direction, deflection_mm, limits),
    }.map_err(make)?;
    let wall = if tilted {
        let side = (dot * length).signum();
        let remove_point = std::array::from_fn(|i| q[i] + b[i]*side);
        trim_surface_by_plane(&wall, q, b, remove_point, deflection_mm, limits)?
    } else { wall };
    total_edges = total_edges.checked_add(edges).ok_or_else(|| EditingError::Budget("Boss edge count".into()))?;
    cut = next.clone(); walls.push(wall);
    }
    let mut joined = cut;
    for wall in &walls { joined = sew(&joined, wall, 1e-7, deflection_mm, limits)?; }
    let m = measure(&joined, limits)?;
    let expected_faces = total_edges.checked_add(original.faces).ok_or_else(|| EditingError::Budget("Boss face count".into()))?;
    let expected_naked = total_edges.checked_add(original.naked_edges).ok_or_else(|| EditingError::Budget("Boss naked edge count".into()))?;
    if m.solids != 0 || m.shells != 1 || m.faces != expected_faces || m.naked_edges != expected_naked {
        return Err(EditingError::Invalid(format!("Boss walls did not join the boundary: {m:?}")));
    }
    Ok(joined)
}

/* ------------------------- split ------------------------- */

/// A split result: the pieces in one compound plus the number of face pieces OCCT produced.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitResult {
    pub brep: Brep,
    pub pieces: usize,
}

/// Form bounded solid regions from intersecting BRep faces or polysurfaces.
/// `None` means that no bounded solid was found. The result is an owned
/// compound with snapshot-local solid indices; publication belongs to the host.
pub fn create_solid_regions(
    inputs: &[Brep],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Option<SplitResult>, EditingError> {
    if inputs.is_empty() || inputs.len() as u64 > limits.max_objects {
        return Err(EditingError::Invalid(
            "create solid needs bounded surface inputs".into(),
        ));
    }
    let deflection = checked_deflection(deflection_mm)?;
    let mut pointers = Vec::new();
    let mut lengths = Vec::new();
    pointers
        .try_reserve_exact(inputs.len())
        .map_err(|_| EditingError::Budget("create solid inputs".into()))?;
    lengths
        .try_reserve_exact(inputs.len())
        .map_err(|_| EditingError::Budget("create solid inputs".into()))?;
    let mut held = om9_core::budget::Usage::array::<*const u8>(pointers.capacity())
        .map_err(|e| EditingError::Budget(e.to_string()))?;
    held.add(
        om9_core::budget::Usage::array::<usize>(lengths.capacity())
            .map_err(|e| EditingError::Budget(e.to_string()))?,
        limits,
    )
    .map_err(|e| EditingError::Budget(e.to_string()))?;
    for input in inputs {
        let m = checked_brep(input, limits)?;
        if m.faces == 0 {
            return Err(EditingError::Invalid("create solid needs faces".into()));
        }
        held.add(
            input
                .validate(limits)
                .map_err(|e| EditingError::Invalid(e.to_string()))?,
            limits,
        )
        .map_err(|e| EditingError::Budget(e.to_string()))?;
        pointers.push(input.source().as_ptr());
        lengths.push(input.source().len());
    }
    let remaining = limits
        .remaining_after(held)
        .map_err(|e| EditingError::Budget(e.to_string()))?;
    let budget = snapshot_budget(&remaining);
    let mut regions = 0;
    let body = snapshot_via_optional(
        deflection,
        &remaining,
        |sink, context, message, capacity| {
            // SAFETY: pointers refer to inputs that outlive this synchronous native call.
            unsafe {
                om9_occt_create_solid_regions(
                    pointers.as_ptr(),
                    lengths.as_ptr(),
                    inputs.len(),
                    &mut regions,
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
    .map_err(EditingError::from)?;
    let Some(brep) = body else { return Ok(None) };
    let pieces = usize::try_from(regions).unwrap_or(0);
    let m = checked_brep(&brep, &remaining)?;
    if pieces == 0
        || pieces as u64 > limits.max_outputs
        || m.solids != pieces
        || !m.closed
        || m.volume_mm3 <= 0.0
    {
        return Err(EditingError::Invalid(
            "create solid region topology or output limit".into(),
        ));
    }
    Ok(Some(SplitResult { brep, pieces }))
}

/// Partition one closed solid by one closed solid, returning its pieces as a
/// compound. This low-level operation does not implement BooleanSplit's
/// multiple selection sets, output objects, options, or document lifecycle.
pub fn split_solid(
    target: &Brep,
    cutter: &Brep,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<SplitResult, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let original = checked_brep(target, limits)?;
    let tool = checked_brep(cutter, limits)?;
    if original.solids != 1 || tool.solids != 1 || !original.closed || !tool.closed {
        return Err(EditingError::Invalid(
            "split needs two closed single solids".into(),
        ));
    }
    let mut pieces = 0;
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: both source buffers and all outputs remain live for the synchronous call.
        unsafe {
            om9_occt_split_solid(
                target.source().as_ptr(),
                target.source().len(),
                cutter.source().as_ptr(),
                cutter.source().len(),
                &mut pieces,
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
    })
    .map_err(EditingError::from)?;
    let pieces = usize::try_from(pieces).unwrap_or(0);
    let result = checked_brep(&brep, limits)?;
    if pieces == 0
        || result.solids != pieces
        || !result.closed
        || (result.volume_mm3 - original.volume_mm3).abs() > original.volume_mm3.abs() * 1e-6 + 1e-9
    {
        return Err(EditingError::Invalid(
            "split solid failed topology or volume validation".into(),
        ));
    }
    Ok(SplitResult { brep, pieces })
}

/// Copy one solid piece out of a split compound. The index is local to this
/// exact immutable compound and is not a persistent subobject identifier.
pub fn split_solid_piece(
    split: &SplitResult,
    index: usize,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    if index >= split.pieces {
        return Err(EditingError::Invalid("split solid piece index".into()));
    }
    let measured = checked_brep(&split.brep, limits)?;
    if measured.solids != split.pieces || !measured.closed {
        return Err(EditingError::Invalid(
            "split solid compound mismatch".into(),
        ));
    }
    let index = c_int::try_from(index).map_err(|_| EditingError::Budget("piece index".into()))?;
    let deflection = checked_deflection(deflection_mm)?;
    let budget = snapshot_budget(limits);
    let piece = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source, callback context and outputs remain live for the synchronous call.
        unsafe {
            om9_occt_split_solid_piece(
                split.brep.source().as_ptr(),
                split.brep.source().len(),
                index,
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
    })
    .map_err(EditingError::from)?;
    let measured = checked_brep(&piece, limits)?;
    if measured.solids != 1 || !measured.closed || measured.volume_mm3 <= 0.0 {
        return Err(EditingError::Invalid(
            "split piece is not a closed solid".into(),
        ));
    }
    Ok(piece)
}

/// Partition the union of closed single-solid targets by all targets and
/// cutters. Equivalent cells are emitted once, even if target volumes overlap
/// or an identical object appears in both sets. This is a geometry helper:
/// document identity, selection and transaction semantics belong to the host.
pub fn split_solid_sets(
    targets: &[Brep],
    cutters: &[Brep],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Vec<Brep>, EditingError> {
    use om9_core::budget::Usage;
    if targets.is_empty() || cutters.is_empty() {
        return Err(EditingError::Invalid(
            "split needs target and cutter sets".into(),
        ));
    }
    for input in targets.iter().chain(cutters) {
        let m = checked_brep(input, limits)?;
        if m.solids != 1 || !m.closed {
            return Err(EditingError::Invalid(
                "split needs closed single solids".into(),
            ));
        }
    }
    let mut results: Vec<Brep> = Vec::new();
    let mut retained = Usage::default();
    for target in targets {
        let mut pending = vec![target.clone()];
        for cutter in targets.iter().chain(cutters) {
            if cutter == target {
                continue;
            }
            let mut next = Vec::new();
            let mut next_usage = retained;
            for candidate in pending {
                let split = split_solid(&candidate, cutter, deflection_mm, limits)?;
                let result_count = next
                    .len()
                    .checked_add(split.pieces)
                    .ok_or_else(|| EditingError::Budget("split output count".into()))?;
                if result_count as u64 > limits.max_outputs {
                    return Err(EditingError::Budget("split output count".into()));
                }
                if split.pieces == 1 {
                    next_usage
                        .add(
                            candidate
                                .validate(limits)
                                .map_err(|e| EditingError::Invalid(e.to_string()))?,
                            limits,
                        )
                        .map_err(|e| EditingError::Budget(e.to_string()))?;
                    next.try_reserve(1)
                        .map_err(|_| EditingError::Budget("split pieces".into()))?;
                    next.push(candidate);
                } else {
                    for index in 0..split.pieces {
                        let piece = split_solid_piece(&split, index, deflection_mm, limits)?;
                        next_usage
                            .add(
                                piece
                                    .validate(limits)
                                    .map_err(|e| EditingError::Invalid(e.to_string()))?,
                                limits,
                            )
                            .map_err(|e| EditingError::Budget(e.to_string()))?;
                        next.try_reserve(1)
                            .map_err(|_| EditingError::Budget("split pieces".into()))?;
                        next.push(piece);
                    }
                }
                if next.len() as u64 > limits.max_outputs {
                    return Err(EditingError::Budget("split output count".into()));
                }
            }
            pending = next;
        }
        for candidate in pending {
            let mut duplicate = false;
            for existing in &results {
                let forward = boolean(
                    &candidate,
                    existing,
                    BooleanOp::Difference,
                    deflection_mm,
                    limits,
                )?;
                if matches!(forward, BooleanOutcome::Empty)
                    && matches!(
                        boolean(
                            existing,
                            &candidate,
                            BooleanOp::Difference,
                            deflection_mm,
                            limits
                        )?,
                        BooleanOutcome::Empty
                    )
                {
                    duplicate = true;
                    break;
                }
            }
            if !duplicate {
                if results.len() as u64 >= limits.max_outputs {
                    return Err(EditingError::Budget("split output count".into()));
                }
                retained
                    .add(
                        candidate
                            .validate(limits)
                            .map_err(|e| EditingError::Invalid(e.to_string()))?,
                        limits,
                    )
                    .map_err(|e| EditingError::Budget(e.to_string()))?;
                results
                    .try_reserve(1)
                    .map_err(|_| EditingError::Budget("split outputs".into()))?;
                results.push(candidate);
            }
        }
    }
    Ok(results)
}

/// Split one exact planar face with a straight trim line that crosses it.
///
/// The sum of the piece areas must equal the original face area; a trim that does not cross the
/// face is refused rather than returning the unsplit face as a "split".
pub fn split_face(
    face: &Brep,
    trim_line: &[Point3],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<SplitResult, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let original = checked_brep(face, limits)?;
    if original.faces != 1 || original.solids != 0 {
        return Err(EditingError::Invalid(
            "split requires exactly one face".into(),
        ));
    }
    if trim_line.len() < 2 {
        return Err(EditingError::Invalid(
            "trim line needs at least two points".into(),
        ));
    }
    let mut flat = Vec::with_capacity(trim_line.len() * 3);
    for point in trim_line {
        if point.iter().any(|value| !value.is_finite()) {
            return Err(EditingError::Invalid(
                "trim line contains a non-finite coordinate".into(),
            ));
        }
        flat.extend_from_slice(point);
    }
    for pair in trim_line.windows(2) {
        let delta = [
            pair[1][0] - pair[0][0],
            pair[1][1] - pair[0][1],
            pair[1][2] - pair[0][2],
        ];
        if delta.iter().map(|value| value * value).sum::<f64>().sqrt() <= 1.0e-9 {
            return Err(EditingError::Invalid("trim line repeats a point".into()));
        }
    }
    let mut pieces: c_int = 0;
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source, trim buffer, piece output and callback context live through this call.
        unsafe {
            om9_occt_split_face(
                face.source().as_ptr(),
                face.source().len(),
                flat.as_ptr(),
                trim_line.len(),
                &mut pieces,
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
    })
    .map_err(EditingError::from)?;
    let pieces = usize::try_from(pieces).unwrap_or(0);
    if pieces < 2 {
        return Err(EditingError::Invalid(
            "trim line does not split the face".into(),
        ));
    }
    let split = measure(&brep, limits)?;
    if split.faces != pieces {
        return Err(EditingError::Invalid(
            "split piece count does not match the face count".into(),
        ));
    }
    if (split.area_mm2 - original.area_mm2).abs() > original.area_mm2 * 1.0e-6 + 1.0e-9 {
        return Err(EditingError::Invalid(
            "split changed the total face area".into(),
        ));
    }
    Ok(SplitResult { brep, pieces })
}

/* ------------------------- cap ------------------------- */

/// Cap supported planar boundary loops, retaining any non-planar openings.
pub fn cap_planar(
    open: &Brep,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(tolerance_mm, "cap tolerance")?;
    let before = checked_brep(open, limits)?;
    if before.closed {
        return Err(EditingError::Invalid(
            "cap requires an open shell, not a closed solid".into(),
        ));
    }
    if before.naked_edges == 0 {
        return Err(EditingError::Invalid(
            "cap requires a naked boundary".into(),
        ));
    }
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_cap(
                open.source().as_ptr(),
                open.source().len(),
                tolerance_mm,
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
    })
    .map_err(EditingError::from)?;
    let after = measure(&brep, limits)?;
    if after.naked_edges >= before.naked_edges || after.faces <= before.faces {
        return Err(EditingError::Kernel("cap did not close a boundary".into()));
    }
    if after.closed && (after.solids == 0 || after.naked_edges != 0) {
        return Err(EditingError::Kernel(
            "cap produced an invalid closed result".into(),
        ));
    }
    Ok(brep)
}

/* ------------------------- fillet and chamfer ------------------------- */

fn edge_edit(
    solid: &Brep,
    edges: &[usize],
    value: f64,
    chamfer: bool,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(
        value,
        if chamfer {
            "chamfer distance"
        } else {
            "fillet radius"
        },
    )?;
    let measures = checked_brep(solid, limits)?;
    if measures.solids == 0 {
        return Err(EditingError::Invalid(
            "edge edit requires a closed solid".into(),
        ));
    }
    let table = edge_table(edges, solid.display.edges.len())?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source, edge table and callback context live through this call.
        unsafe {
            if chamfer {
                om9_occt_chamfer(
                    solid.source().as_ptr(),
                    solid.source().len(),
                    table.as_ptr(),
                    table.len(),
                    value,
                    deflection,
                    budget.points,
                    budget.nodes,
                    budget.bytes,
                    sink,
                    context,
                    message,
                    capacity,
                )
            } else {
                om9_occt_fillet(
                    solid.source().as_ptr(),
                    solid.source().len(),
                    table.as_ptr(),
                    table.len(),
                    value,
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
        }
    })
    .map_err(EditingError::from)
}

/// Round the selected edges of a solid.
pub fn fillet(
    solid: &Brep,
    edges: &[usize],
    radius_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    edge_edit(solid, edges, radius_mm, false, deflection_mm, limits)
}

/// Apply one constant radius per selected edge. Differing values on one
/// automatically propagated contour are rejected by the native adapter.
pub fn fillet_with_radii(
    solid: &Brep,
    edges: &[(usize, f64)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let measures = checked_brep(solid, limits)?;
    if measures.faces < 2 {
        return Err(EditingError::Invalid(
            "fillet requires at least two joined faces".into(),
        ));
    }
    if edges.is_empty() {
        return Err(EditingError::Invalid("no edge selected".into()));
    }
    let mut indices = Vec::new();
    let mut radii = Vec::new();
    indices
        .try_reserve_exact(edges.len())
        .map_err(|_| EditingError::Budget("fillet indices".into()))?;
    radii
        .try_reserve_exact(edges.len())
        .map_err(|_| EditingError::Budget("fillet radii".into()))?;
    for &(index, radius) in edges {
        positive(radius, "fillet radius")?;
        let native =
            i32::try_from(index).map_err(|_| EditingError::Budget("fillet edge index".into()))?;
        if index >= solid.display.edges.len() || indices.contains(&native) {
            return Err(EditingError::Invalid(
                "invalid or duplicate fillet edge".into(),
            ));
        }
        indices.push(native);
        radii.push(radius);
    }
    let budget = snapshot_budget(limits);
    snapshot_via(
        deflection,
        limits,
        |sink, context, message, capacity| unsafe {
            // SAFETY: source, both equal-sized slices and callback context outlive the call.
            om9_occt_fillet_radii(
                solid.source().as_ptr(),
                solid.source().len(),
                indices.as_ptr(),
                radii.as_ptr(),
                indices.len(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        },
    )
    .map_err(EditingError::from)
}

/// Retrieve exact endpoints in the orientation used by `fillet_with_end_radii`.
pub fn straight_edge_endpoints(shape: &Brep, index: usize, limits: &Limits) -> Result<(Point3, Point3), EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.display.edges.len() { return Err(EditingError::Invalid("edge index out of range".into())); }
    let edge = i32::try_from(index).map_err(|_| EditingError::Budget("edge index".into()))?;
    let mut is_line = 0;
    let mut message = [0 as c_char; 512];
    // SAFETY: source and output buffers remain alive for the synchronous call.
    let status = unsafe { om9_occt_edge_is_line(shape.source().as_ptr(), shape.source().len(), edge,
        &mut is_line, message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        // SAFETY: the zero-initialized buffer remains NUL terminated.
        let detail = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned();
        return Err(EditingError::Kernel(detail));
    }
    if is_line == 0 { return Err(EditingError::Invalid("edge is not a straight line".into())); }
    oriented_edge_endpoints(shape, index, limits)
}

/// Retrieve exact endpoints in the orientation used by `fillet_with_end_radii`.
pub fn oriented_edge_endpoints(
    shape: &Brep,
    index: usize,
    limits: &Limits,
) -> Result<(Point3, Point3), EditingError> {
    oriented_edge_endpoints_or_closed(shape, index, limits)?
        .ok_or_else(|| EditingError::Invalid("edge has a single closed endpoint".into()))
}

/// Return oriented endpoints, or None only for an edge with one shared topological vertex.
pub fn oriented_edge_endpoints_or_closed(
    shape: &Brep,
    index: usize,
    limits: &Limits,
) -> Result<Option<(Point3, Point3)>, EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.display.edges.len() {
        return Err(EditingError::Invalid("edge index out of range".into()));
    }
    let index = i32::try_from(index).map_err(|_| EditingError::Budget("edge index".into()))?;
    let mut points = [0.0_f64; 6];
    let mut message = [0 as c_char; 512];
    // SAFETY: buffers and immutable BRep source remain alive through the call.
    let status = unsafe {
        om9_occt_oriented_edge_endpoints(
            shape.source().as_ptr(),
            shape.source().len(),
            index,
            points.as_mut_ptr(),
            message.as_mut_ptr(),
            message.len() as c_int,
        )
    };
    if status == 3 {
        return Ok(None);
    }
    if status != 0 {
        // SAFETY: the zero-initialized buffer remains NUL terminated.
        let detail = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        return Err(EditingError::Kernel(detail));
    }
    if !points.iter().all(|value| value.is_finite()) {
        return Err(EditingError::Kernel("non-finite edge endpoints".into()));
    }
    Ok(Some((
        [points[0], points[1], points[2]],
        [points[3], points[4], points[5]],
    )))
}

/// Extract a bounded circular edge in BRep-local coordinates, preserving edge orientation.
pub fn circular_edge(shape: &Brep, index: usize, limits: &Limits) -> Result<om9_core::geometry::Curve, EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.display.edges.len() { return Err(EditingError::Invalid("edge index out of range".into())); }
    let index = i32::try_from(index).map_err(|_| EditingError::Budget("edge index".into()))?;
    let mut values = [0.0; 13];
    let mut message = [0 as c_char; 512];
    // SAFETY: all fixed buffers and BRep bytes stay alive during the synchronous native query.
    let status = unsafe { om9_occt_circular_edge(shape.source().as_ptr(), shape.source().len(), index,
        values.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    if !values.iter().all(|v| v.is_finite()) || values[9] <= 0.0 || values[12] != 0.0 && values[12] != 1.0 {
        return Err(EditingError::Kernel("invalid circular edge data".into()));
    }
    let reversed = values[12] == 1.0;
    let start = if reversed { values[11] } else { values[10] };
    let span = values[11] - values[10];
    let (sin, cos) = start.sin_cos();
    let x = std::array::from_fn(|i| values[3+i] * cos + values[6+i] * sin);
    let y = std::array::from_fn(|i| (values[6+i] * cos - values[3+i] * sin) * if reversed { -1.0 } else { 1.0 });
    let curve = om9_core::geometry::Curve::Circle { center: [values[0], values[1], values[2]], x, y,
        radius: values[9], domain: [0.0, span], angle_domain: None };
    curve.validate(limits).map_err(|error| EditingError::Invalid(format!("circular edge: {error}")))?;
    Ok(curve)
}

/// Project a point onto an open edge, returning oriented arc-length fraction and exact point.
pub fn fillet_project_edge(shape: &Brep, edge: usize, point: Point3, limits: &Limits) -> Result<(f64, Point3), EditingError> {
    checked_brep(shape, limits)?;
    if edge >= shape.display.edges.len() || !point.iter().all(|x| x.is_finite()) {
        return Err(EditingError::Invalid("fillet edge projection input".into()));
    }
    let edge = i32::try_from(edge).map_err(|_| EditingError::Budget("fillet edge index".into()))?;
    let mut output = [0.0; 4];
    let mut message = [0 as c_char; 512];
    // SAFETY: the source, point and fixed output buffers remain alive for this synchronous call.
    let status = unsafe { om9_occt_fillet_project_edge(shape.source().as_ptr(), shape.source().len(), edge,
        point.as_ptr(), output.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        // SAFETY: message is initialized and NUL terminated.
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    if !output.iter().all(|v| v.is_finite()) || !(0.0..=1.0).contains(&output[0]) {
        return Err(EditingError::Kernel("invalid projected fillet point".into()));
    }
    Ok((output[0], [output[1], output[2], output[3]]))
}

/// Identify candidate oriented endpoints sharing the anchor's vertex and tangent contour.
pub fn fillet_shared_endpoint(shape: &Brep, anchor: usize, side: usize, candidate: usize, limits: &Limits) -> Result<u8, EditingError> {
    Ok(fillet_shared_endpoints(shape, anchor, side, &[candidate], limits)?[0])
}

/// Classify a batch against one contour, reading and validating the source only once.
pub fn fillet_shared_endpoints(shape: &Brep, anchor: usize, side: usize, candidates: &[usize], limits: &Limits) -> Result<Vec<u8>, EditingError> {
    checked_brep(shape, limits)?;
    if anchor >= shape.display.edges.len() || candidates.is_empty() || candidates.iter().any(|&i| i >= shape.display.edges.len()) || side > 1 {
        return Err(EditingError::Invalid("invalid fillet endpoint".into()));
    }
    let bytes = candidates.len().checked_mul(std::mem::size_of::<c_int>() + std::mem::size_of::<u8>())
        .ok_or_else(|| EditingError::Budget("fillet endpoint arrays".into()))?;
    if candidates.len() as u64 > limits.max_outputs || bytes as u64 > limits.max_bytes {
        return Err(EditingError::Budget("fillet endpoint arrays".into()));
    }
    let anchor = i32::try_from(anchor).map_err(|_| EditingError::Budget("fillet edge index".into()))?;
    let mut native = Vec::new();
    native.try_reserve_exact(candidates.len()).map_err(|_| EditingError::Budget("fillet edge indices".into()))?;
    for &candidate in candidates { native.push(i32::try_from(candidate).map_err(|_| EditingError::Budget("fillet edge index".into()))?); }
    let mut masks = Vec::new();
    masks.try_reserve_exact(candidates.len()).map_err(|_| EditingError::Budget("fillet endpoint masks".into()))?;
    masks.resize(candidates.len(), 0);
    let mut message = [0 as c_char; 512];
    // SAFETY: immutable source and fixed output buffers remain valid for this synchronous call.
    let status = unsafe { om9_occt_fillet_shared_endpoints(shape.source().as_ptr(), shape.source().len(), anchor,
        side as c_int, native.as_ptr(), native.len(), masks.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        // SAFETY: the buffer is initialized with a terminating NUL.
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    if masks.iter().any(|&mask| mask > 3) { return Err(EditingError::Kernel("invalid endpoint mask".into())); }
    Ok(masks)
}

/// Fillet selected edges with a linearly changing radius between their oriented endpoints.
/// Variable values on an open tangent contour require all of its edges and matching radii
/// at shared vertices; the native kernel interpolates the supplied contour law.
pub fn fillet_with_end_radii(
    shape: &Brep,
    edges: &[(usize, f64, f64)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    fillet_with_radius_controls(shape, edges, &[], deflection_mm, limits)
}

/// Radius controls use an edge index, oriented arc-length fraction in (0,1), and radius.
/// The contour must be open and fully selected; OCCT interpolates the supplied law.
pub fn fillet_with_radius_controls(
    shape: &Brep,
    edges: &[(usize, f64, f64)],
    controls: &[(usize, f64, f64)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let measures = checked_brep(shape, limits)?;
    if measures.faces < 2 {
        return Err(EditingError::Invalid(
            "fillet requires at least two joined faces".into(),
        ));
    }
    if edges.is_empty() {
        return Err(EditingError::Invalid("no edge selected".into()));
    }
    if edges.len() as u64 > limits.max_outputs || controls.len() as u64 > limits.max_outputs {
        return Err(EditingError::Budget("fillet control count".into()));
    }
    let input_bytes = edges.len().checked_add(controls.len())
        .and_then(|n| n.checked_mul(std::mem::size_of::<c_int>() + 2 * std::mem::size_of::<f64>()))
        .ok_or_else(|| EditingError::Budget("fillet control arrays".into()))?;
    if input_bytes as u64 > limits.max_bytes { return Err(EditingError::Budget("fillet control arrays".into())); }
    let mut indices = Vec::new();
    let mut starts = Vec::new();
    let mut ends = Vec::new();
    indices
        .try_reserve_exact(edges.len())
        .map_err(|_| EditingError::Budget("fillet indices".into()))?;
    starts
        .try_reserve_exact(edges.len())
        .map_err(|_| EditingError::Budget("fillet radii".into()))?;
    ends.try_reserve_exact(edges.len())
        .map_err(|_| EditingError::Budget("fillet radii".into()))?;
    for &(index, start, end) in edges {
        positive(start, "fillet start radius")?;
        positive(end, "fillet end radius")?;
        let native =
            i32::try_from(index).map_err(|_| EditingError::Budget("fillet edge index".into()))?;
        if index >= shape.display.edges.len() || indices.contains(&native) {
            return Err(EditingError::Invalid(
                "invalid or duplicate fillet edge".into(),
            ));
        }
        indices.push(native);
        starts.push(start);
        ends.push(end);
    }
    let mut control_edges = Vec::new();
    let mut fractions = Vec::new();
    let mut control_radii = Vec::new();
    control_edges.try_reserve_exact(controls.len()).map_err(|_| EditingError::Budget("fillet controls".into()))?;
    fractions.try_reserve_exact(controls.len()).map_err(|_| EditingError::Budget("fillet controls".into()))?;
    control_radii.try_reserve_exact(controls.len()).map_err(|_| EditingError::Budget("fillet controls".into()))?;
    for (i, &(edge, fraction, radius)) in controls.iter().enumerate() {
        positive(radius, "fillet control radius")?;
        if !fraction.is_finite() || !(0.0..1.0).contains(&fraction) || fraction == 0.0 ||
            !edges.iter().any(|item| item.0 == edge) || controls[..i].iter().any(|other|
                other.0 == edge && (other.1 - fraction).abs() < 1e-9) {
            return Err(EditingError::Invalid("invalid or duplicate fillet control".into()));
        }
        control_edges.push(i32::try_from(edge).map_err(|_| EditingError::Budget("fillet edge index".into()))?);
        fractions.push(fraction);
        control_radii.push(radius);
    }
    let budget = snapshot_budget(limits);
    snapshot_via(
        deflection,
        limits,
        |sink, context, message, capacity| unsafe {
            // SAFETY: source, equal-sized radius/index slices and callback context outlive the call.
            om9_occt_fillet_end_radii(
                shape.source().as_ptr(),
                shape.source().len(),
                indices.as_ptr(),
                starts.as_ptr(),
                ends.as_ptr(),
                indices.len(),
                control_edges.as_ptr(),
                fractions.as_ptr(),
                control_radii.as_ptr(),
                controls.len(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        },
    )
    .map_err(EditingError::from)
}

/// Chamfer the selected edges of a solid.
pub fn chamfer(
    solid: &Brep,
    edges: &[usize],
    distance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    edge_edit(solid, edges, distance_mm, true, deflection_mm, limits)
}

/// The two endpoints of one published edge, from the display snapshot (selection helper only).
///
/// The acceptance measurements never use this; it exists so a caller can name an edge by geometry
/// before passing its index to [`fillet`] or [`chamfer`].
pub fn edge_endpoints(brep: &Brep, edge_index: usize) -> Result<(Point3, Point3), EditingError> {
    if edge_index >= brep.display.edges.len() {
        return Err(EditingError::Invalid(format!(
            "edge index {edge_index} is outside the {} published edges",
            brep.display.edges.len()
        )));
    }
    let mut first: Option<Point3> = None;
    let mut last: Option<Point3> = None;
    for [a, b, edge] in &brep.display.segments {
        if *edge as usize != edge_index {
            continue;
        }
        if first.is_none() {
            first = Some(brep.display.points[*a as usize]);
        }
        last = Some(brep.display.points[*b as usize]);
    }
    match (first, last) {
        (Some(first), Some(last)) => Ok((first, last)),
        _ => Err(EditingError::Invalid(
            "the edge has no display segment".into(),
        )),
    }
}

/* ------------------------- offset and thicken ------------------------- */

/// Thicken a face or shell into a closed solid of the given one-sided thickness.
pub fn thicken(
    shape: &Brep,
    thickness_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(thickness_mm, "thickness")?;
    let before = checked_brep(shape, limits)?;
    if before.solids != 0 {
        return Err(EditingError::Invalid(
            "thicken requires a face or shell".into(),
        ));
    }
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_thicken(
                shape.source().as_ptr(),
                shape.source().len(),
                thickness_mm,
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
    })
    .map_err(EditingError::from)?;
    let after = measure(&brep, limits)?;
    if after.solids == 0 || !after.closed {
        return Err(EditingError::Kernel(
            "thicken did not produce a closed solid".into(),
        ));
    }
    Ok(brep)
}

/// Offset a face or shell by a signed distance, preserving its face count and area.
pub fn offset_shape(
    shape: &Brep,
    distance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    if !distance_mm.is_finite() || distance_mm == 0.0 {
        return Err(EditingError::Invalid(
            "offset distance must be finite and non-zero".into(),
        ));
    }
    let before = checked_brep(shape, limits)?;
    if before.faces == 0 {
        return Err(EditingError::Invalid(
            "offset requires a face or shell".into(),
        ));
    }
    let budget = snapshot_budget(limits);
    let brep = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_offset_shape(
                shape.source().as_ptr(),
                shape.source().len(),
                distance_mm,
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
    })
    .map_err(EditingError::from)?;
    let after = measure(&brep, limits)?;
    if after.faces != before.faces {
        return Err(EditingError::Kernel(
            "offset changed the face count of the input".into(),
        ));
    }
    Ok(brep)
}

/* ------------------------- extract, unjoin, rejoin ------------------------- */

/// Extract one face of a body as its own owned B-Rep. The source body is untouched.
pub fn extract_face(
    brep: &Brep,
    face_index: usize,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    if face_index >= before.faces {
        return Err(EditingError::Invalid(format!(
            "face index {face_index} is outside the {} faces",
            before.faces
        )));
    }
    let native_index =
        i32::try_from(face_index).map_err(|_| EditingError::Budget("face index".into()))?;
    let budget = snapshot_budget(limits);
    let face = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_extract_face(
                brep.source().as_ptr(),
                brep.source().len(),
                native_index,
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
    })
    .map_err(EditingError::from)?;
    let after = measure(&face, limits)?;
    if after.faces != 1 {
        return Err(EditingError::Kernel(
            "extract did not produce exactly one face".into(),
        ));
    }
    Ok(face)
}

/// Remove one face, leaving the remaining faces as an open shell (unjoin).
pub fn remove_face(
    brep: &Brep,
    face_index: usize,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    if face_index >= before.faces {
        return Err(EditingError::Invalid(format!(
            "face index {face_index} is outside the {} faces",
            before.faces
        )));
    }
    let native_index =
        i32::try_from(face_index).map_err(|_| EditingError::Budget("face index".into()))?;
    let budget = snapshot_budget(limits);
    let shell = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_remove_face(
                brep.source().as_ptr(),
                brep.source().len(),
                native_index,
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
    })
    .map_err(EditingError::from)?;
    let after = measure(&shell, limits)?;
    if after.faces + 1 != before.faces {
        return Err(EditingError::Kernel(
            "remove face did not keep exactly the other faces".into(),
        ));
    }
    if after.closed {
        return Err(EditingError::Kernel(
            "remove face unexpectedly stayed closed".into(),
        ));
    }
    Ok(shell)
}

/// Remove faces by their indices in the original B-Rep; the remainder must be nonempty.
pub fn remove_faces(
    brep: &Brep,
    face_indices: &[usize],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    if face_indices.is_empty() || face_indices.len() >= before.faces {
        return Err(EditingError::Invalid(
            "remove faces requires a nonempty remainder".into(),
        ));
    }
    let mut indices = Vec::new();
    indices
        .try_reserve_exact(face_indices.len())
        .map_err(|_| EditingError::Budget("face indices".into()))?;
    for &index in face_indices {
        if index >= before.faces {
            return Err(EditingError::Invalid("face index out of range".into()));
        }
        indices.push(i32::try_from(index).map_err(|_| EditingError::Budget("face index".into()))?);
    }
    indices.sort_unstable();
    if indices.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(EditingError::Invalid("duplicate face index".into()));
    }
    let budget = snapshot_budget(limits);
    let shell = snapshot_via(
        deflection,
        limits,
        |sink, context, message, capacity| unsafe {
            // SAFETY: source, index slice, and callback context live through the call.
            om9_occt_remove_faces(
                brep.source().as_ptr(),
                brep.source().len(),
                indices.as_ptr(),
                indices.len(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        },
    )
    .map_err(EditingError::from)?;
    let after = measure(&shell, limits)?;
    if after.faces + face_indices.len() != before.faces || after.closed {
        return Err(EditingError::Kernel(
            "remove faces produced an invalid remainder".into(),
        ));
    }
    Ok(shell)
}

/// Sew two B-Rep parts together. A closed sewn shell becomes a solid; an open join stays a shell.
pub fn sew(
    a: &Brep,
    b: &Brep,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    sew_with_orientation(a, b, false, tolerance_mm, deflection_mm, limits)
}

/// Sew shapes while reversing the second shape's topology before sewing.
pub fn sew_reversing_second(
    a: &Brep, b: &Brep, tolerance_mm: f64, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    sew_with_orientation(a, b, true, tolerance_mm, deflection_mm, limits)
}

fn sew_with_orientation(
    a: &Brep, b: &Brep, reverse_b: bool, tolerance_mm: f64,
    deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(tolerance_mm, "sew tolerance")?;
    checked_brep(a, limits)?;
    checked_brep(b, limits)?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: both sources and the callback context live through this call.
        unsafe {
            om9_occt_sew(
                a.source().as_ptr(),
                a.source().len(),
                b.source().as_ptr(),
                b.source().len(),
                i32::from(reverse_b),
                tolerance_mm,
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
    })
    .map_err(EditingError::from)
}

/// Sew a list of parts in order (rejoin). The intermediate result is folded pairwise.
pub fn sew_all(
    parts: &[Brep],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    if parts.len() < 2 {
        return Err(EditingError::Invalid(
            "join needs at least two parts".into(),
        ));
    }
    positive(tolerance_mm, "sew tolerance")?;
    let mut joined = sew(&parts[0], &parts[1], tolerance_mm, deflection_mm, limits)?;
    for part in &parts[2..] {
        joined = sew(&joined, part, tolerance_mm, deflection_mm, limits)?;
    }
    Ok(joined)
}

/* ------------------------- health ------------------------- */

/// Exact B-Rep health. Naked edges are free boundary edges of the exact topology; the tessellation
/// is never consulted, so a mesh boundary can never masquerade as a B-Rep result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryHealth {
    pub valid: bool,
    pub closed: bool,
    pub naked_edges: usize,
    pub faces: usize,
    pub edges: usize,
    pub vertices: usize,
    pub solids: usize,
    pub shells: usize,
    pub source_sha256: [u8; 32],
}

impl GeometryHealth {
    pub fn is_solid(&self) -> bool {
        self.solids > 0 && self.closed
    }
}

/// Report health for one canonical B-Rep source.
pub fn geometry_health(brep: &Brep, limits: &Limits) -> Result<GeometryHealth, EditingError> {
    let measures = checked_brep(brep, limits)?;
    Ok(GeometryHealth {
        valid: true,
        closed: measures.closed,
        naked_edges: measures.naked_edges,
        faces: measures.faces,
        edges: measures.edges,
        vertices: measures.vertices,
        solids: measures.solids,
        shells: measures.shells,
        source_sha256: brep.display.source_sha256,
    })
}

unsafe extern "C" {
    fn om9_occt_shell(
        source: *const u8,
        length: usize,
        face_indices: *const usize,
        face_count: usize,
        thickness: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}

/// Hollow one solid by removing `face_index` and walling the remainder with `thickness_mm`.
pub fn shell(
    solid: &Brep,
    face_index: usize,
    thickness_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    shell_faces(solid, &[face_index], thickness_mm, deflection_mm, limits)
}

/// Hollow a closed solid, removing the specified distinct faces in one operation.
pub fn shell_faces(
    solid: &Brep,
    face_indices: &[usize],
    thickness_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(thickness_mm, "shell thickness")?;
    let before = checked_brep(solid, limits)?;
    if !before.closed || before.solids != 1 {
        return Err(EditingError::Invalid("shell requires one closed solid".into()));
    }
    if face_indices.is_empty() || face_indices.len() >= before.faces || face_indices.len() as u64 > limits.max_outputs {
        return Err(EditingError::Invalid("shell requires selected and retained faces".into()));
    }
    for (i, &face) in face_indices.iter().enumerate() {
        if face >= before.faces || face_indices[..i].contains(&face) {
            return Err(EditingError::Invalid("invalid or duplicate shell face".into()));
        }
    }
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_shell(
                solid.source().as_ptr(),
                solid.source().len(),
                face_indices.as_ptr(),
                face_indices.len(),
                thickness_mm,
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
    })
    .map_err(EditingError::from)
}
