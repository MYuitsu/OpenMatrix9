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
    fn om9_occt_sew(
        source_a: *const u8,
        length_a: usize,
        source_b: *const u8,
        length_b: usize,
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

/* ------------------------- split ------------------------- */

/// A split result: the pieces in one compound plus the number of face pieces OCCT produced.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitResult {
    pub brep: Brep,
    pub pieces: usize,
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

/// Cap the naked planar loop of an open shell and close it into a solid.
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
    if !after.closed || after.solids == 0 || after.naked_edges != 0 {
        return Err(EditingError::Kernel(
            "cap produced a shape that is still open".into(),
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

/// Sew two B-Rep parts together. A closed sewn shell becomes a solid; an open join stays a shell.
pub fn sew(
    a: &Brep,
    b: &Brep,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
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
        face_index: usize,
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
    let deflection = checked_deflection(deflection_mm)?;
    positive(thickness_mm, "shell thickness")?;
    let before = checked_brep(solid, limits)?;
    if before.faces == 0 || face_index >= before.faces {
        return Err(EditingError::Invalid("shell face index".into()));
    }
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_shell(
                solid.source().as_ptr(),
                solid.source().len(),
                face_index,
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
