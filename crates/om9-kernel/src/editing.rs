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
use om9_core::brep::BrepSourceAccess;
use crate::{
    brep::{snapshot_budget, snapshot_via, snapshot_via_optional, snapshot_via_with_admission},
    creation::{CreationError, Measures, measure},
};
use om9_core::{brep::Brep, budget::Limits, geometry::Point3};
use std::ffi::{c_char, c_int, c_void};

/// The typed refusal contract is shared with the creation owner.
pub type EditingError = CreationError;

type NativeSink = unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int;

unsafe extern "C" {
    fn om9_occt_boolean_source(certified: c_int, a: *const u8, alen: usize, b: *const u8, blen: usize,
        nodes: usize, bytes: usize, sink: crate::brep::intermediate::Sink, context: *mut c_void,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_transform_source(source: *const u8, length: usize, matrix: *const f64,
        nodes: usize, bytes: usize, sink: crate::brep::intermediate::Sink, context: *mut c_void,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_default_face_isocurves(source:*const u8,length:usize,
        max_points:usize,max_nodes:usize,max_bytes:usize,max_degree:c_int,
        checkpoint:unsafe extern "C" fn(*mut c_void)->c_int,
        prepare:unsafe extern "C" fn(*mut c_void,usize)->c_int,
        sink:unsafe extern "C" fn(*mut c_void,c_int,*const f64,*const f64,usize,*const f64,usize)->c_int,
        context:*mut c_void,message:*mut c_char,capacity:c_int)->c_int;
    fn om9_occt_isocurve_face_bounds(source:*const u8,length:usize,max_nodes:usize,triangle:*const f64,
        witness_deflection:f64,tolerance:f64,bounds:*mut f64,
        message:*mut c_char,capacity:c_int)->c_int;
    fn om9_occt_isocurve_ray_face_bounds(source:*const u8,length:usize,max_nodes:usize,origin:*const f64,
        direction:*const f64,reference:*const f64,tolerance:f64,max_distance:f64,witness:*const f64,witness_deflection:f64,witness_tolerance:f64,bounds:*mut f64,
        resolved:*mut f64,message:*mut c_char,capacity:c_int)->c_int;
    fn om9_occt_face_isocurve(source:*const u8,length:usize,along_u:c_int,value:f64,
        triangle:*const f64,witness_deflection:f64,tolerance:f64,
        max_points:usize,max_nodes:usize,max_bytes:usize,max_degree:c_int,
        prepare:unsafe extern "C" fn(*mut c_void,usize)->c_int,
        sink:unsafe extern "C" fn(*mut c_void,c_int,*const f64,*const f64,usize,*const f64,usize)->c_int,
        context:*mut c_void,message:*mut c_char,capacity:c_int)->c_int;
    fn om9_occt_ray_face_isocurve(source:*const u8,length:usize,along_u:c_int,value:f64,
        origin:*const f64,direction:*const f64,reference:*const f64,tolerance:f64,max_distance:f64,
        witness:*const f64,witness_deflection:f64,witness_tolerance:f64,
        max_points:usize,max_nodes:usize,max_bytes:usize,max_degree:c_int,
        prepare:unsafe extern "C" fn(*mut c_void,usize)->c_int,
        sink:unsafe extern "C" fn(*mut c_void,c_int,*const f64,*const f64,usize,*const f64,usize)->c_int,
        context:*mut c_void,message:*mut c_char,capacity:c_int)->c_int;
    fn om9_occt_verify_face_triangles(source: *const u8, length: usize, triangles: *const f64,
        count: usize, tolerance: f64, vertex_tolerance: f64, max_nodes: usize, max_bytes: usize,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_verified_edge_endpoints(source: *const u8, length: usize, segment: *const f64,
        tolerance: f64, segment_tolerance: f64, max_nodes: usize, points: *mut f64,
        message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_all_edges_nurbs(source: *const u8, length: usize, max_edges: usize, boundary_only: c_int,
        max_points: usize, max_nodes: usize, max_bytes: usize, max_degree: c_int,
        prepare: unsafe extern "C" fn(*mut c_void, usize) -> c_int,
        sink: unsafe extern "C" fn(*mut c_void, c_int, *const f64, *const f64, usize, *const f64, usize) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_point_edge_nurbs(source: *const u8, length: usize, point: *const f64, tolerance: f64,
        segment: *const f64, segment_tolerance: f64, resolved_point: *mut f64, circle_data: *mut f64,
        max_edges: usize, max_points: usize, max_nodes: usize, max_bytes: usize, max_degree: c_int,
        sink: unsafe extern "C" fn(*mut c_void, c_int, *const f64, *const f64, usize, *const f64, usize) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_edges_nurbs(source: *const u8, length: usize, indices: *const c_int, count: usize,
        max_points: usize, max_nodes: usize, max_bytes: usize, max_degree: c_int,
        sink: unsafe extern "C" fn(*mut c_void, c_int, *const f64, *const f64, usize, *const f64, usize) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
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
        require_solids: c_int,
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
    fn om9_occt_cut_disjoint_tools(
        source: *const u8, source_len: usize,
        tools: *const *const u8, lengths: *const usize, count: usize,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: NativeSink, context: *mut c_void, message: *mut c_char, capacity: c_int,
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
        segments: *const f64,
        pick_tolerance: f64,
        segment_tolerance: f64,
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
        candidates: *const c_int, count: usize, segments: *const f64, anchor_slot: usize,
        tolerance: f64, segment_tolerance: f64, max_nodes: usize, masks: *mut u8, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_circular_edge(source: *const u8, length: usize, edge: c_int,
        output: *mut f64, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_circular_edge_centers(source: *const u8, length: usize, max_edges: usize, max_outputs: usize,
        checkpoint: unsafe extern "C" fn(*mut c_void) -> c_int,
        sink: unsafe extern "C" fn(*mut c_void, *const f64) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_source_vertices(source: *const u8, length: usize, max_nodes: usize, max_outputs: usize,
        checkpoint: unsafe extern "C" fn(*mut c_void) -> c_int,
        sink: unsafe extern "C" fn(*mut c_void, *const f64) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_fillet_project_edge(source: *const u8, length: usize, edge: c_int,
        point: *const f64, segment: *const f64, tolerance: f64, segment_tolerance: f64, max_nodes: usize,
        output: *mut f64, message: *mut c_char, capacity: c_int) -> c_int;
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
    fn om9_occt_verify_face_display(source: *const u8, length: usize,
        witness: *const f64, deflection: f64, vertex_tolerance: f64,
        points: *const f64, point_count: usize, triangles: *const u32, triangle_count: usize,
        display_face: u32, max_nodes: usize, canceled: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
        cancel_context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_extract_face(
        source: *const u8,
        length: usize,
        face_index: c_int,
        triangle: *const f64,
        vertex_tolerance: f64,
        witness_deflection: f64,
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
        triangle: *const f64,
        vertex_tolerance: f64,
        witness_deflection: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_remove_region(
        source: *const u8,
        length: usize,
        face_index: c_int,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_unify_surface_regions(
        source: *const u8,
        length: usize,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_unify_surface_preserving_edges(
        source: *const u8, length: usize, original_source: *const u8, original_length: usize,
        deflection: f64, max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: NativeSink, context: *mut c_void, message: *mut c_char, capacity: c_int,
    ) -> c_int;
    fn om9_occt_remove_faces(
        source: *const u8,
        length: usize,
        face_indices: *const c_int,
        count: usize,
        triangles: *const f64,
        vertex_tolerance: f64,
        witness_deflection: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_sew_many(
        sources: *const *const u8,
        lengths: *const usize,
        count: usize,
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
    fn om9_occt_sew(
        source_a: *const u8,
        length_a: usize,
        source_b: *const u8,
        length_b: usize,
        reverse_b: c_int,
        preserve_surfaces: c_int,
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

/// Extract the exact native edge curve as an owned rational B-spline. Native
/// OCCT parsing/conversion has its own allocations outside the Rust budget.
pub fn edge_nurbs(shape: &dyn BrepSourceAccess, index: usize, limits: &Limits) -> Result<om9_core::geometry::Curve, EditingError> {
    edges_nurbs(shape, &[index], limits)?.pop().ok_or_else(|| EditingError::Kernel("edge curve output missing".into()))
}

/// Extract selected edges in order while parsing the source only once.
pub fn edges_nurbs(shape: &dyn BrepSourceAccess, indices: &[usize], limits: &Limits) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    edges_nurbs_admitted(shape, indices, limits, |_, _| Ok(()))
}

/// Caller-accounted extraction; the callback covers Rust output and scratch,
/// not allocations made internally by OCCT while parsing/converting curves.
pub fn edges_nurbs_admitted(shape: &dyn BrepSourceAccess, indices: &[usize], limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    edge_curves_admitted(shape, indices, None, None, &mut [0.; 3], None, 0, limits, &mut admit)
}

/// Extract mapped edge indices from an attested, independently retained source.
/// The mapping must have been produced from this registered source's manifest;
/// publication remains the caller's responsibility.
pub fn registered_edges_nurbs_admitted(source: &crate::brep::RegisteredTopologySource, indices: &[usize], limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    edge_curves_raw_admitted(source.source(), Some(source.topology().edges.len()), indices,
        None, None, &mut [0.; 3], None, 0, None, limits, &mut admit)
}

/// Extract source edges with 3D curves from one native map; verified degenerate pole edges are skipped.
pub fn all_source_edges_nurbs_admitted(shape: &dyn BrepSourceAccess, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    edge_curves_admitted(shape, &[], None, None, &mut [0.; 3], None, 1, limits, &mut admit)
}

/// Extract all edges from canonical OCCT source without constructing a display
/// mesh. The caller keeps the source alive and admits it alongside outputs.
pub fn all_edges_from_source_admitted(source:&str, limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>,EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {
        return Err(EditingError::Invalid("edge source".into()));
    }
    edge_curves_raw_admitted(source,None,&[],None,None,&mut [0.;3],None,1,None,limits,&mut admit)
}

/// Like all-edge extraction, but an empty nondegenerate edge set is valid for
/// snapping because topological vertices are captured independently.
pub fn snap_edges_from_source_admitted(source: &str, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes {
        return Err(EditingError::Invalid("edge source".into()));
    }
    edge_curves_raw_admitted(source, None, &[], None, None, &mut [0.; 3], None, 5, None, limits, &mut admit)
}

/// Capture centers from analytic circular source edges. The caller admits the
/// retained source, the returned vector and transient native work separately.
pub fn circular_edge_centers_from_source_admitted(source: &str, limits: &Limits,
    admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<Point3>, EditingError> {
    source_points_from_source_admitted(source, limits, true, admit)
}

/// Capture canonical topological vertices, including vertices on degenerate edges.
pub fn vertices_from_source_admitted(source: &str, limits: &Limits,
    admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<Point3>, EditingError> {
    source_points_from_source_admitted(source, limits, false, admit)
}

fn source_points_from_source_admitted(source: &str, limits: &Limits, circular: bool,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<Point3>, EditingError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes {
        return Err(EditingError::Invalid("edge center source".into()));
    }
    struct Capture<'a> {
        limits: &'a Limits,
        centers: Vec<Point3>,
        error: Option<EditingError>,
        admit: &'a mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
    }
    unsafe extern "C" fn checkpoint(context: *mut c_void) -> c_int {
        let capture = unsafe { &mut *context.cast::<Capture<'_>>() };
        let bytes = (capture.centers.capacity() as u64).saturating_mul(std::mem::size_of::<Point3>() as u64);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (capture.admit)(bytes, 0)))
            .unwrap_or_else(|_| Err(om9_core::Error::Invalid("edge center checkpoint panic")));
        match result { Ok(()) => 0, Err(error) => { capture.error = Some(EditingError::Budget(error.to_string())); 1 } }
    }
    unsafe extern "C" fn sink(context: *mut c_void, xyz: *const f64) -> c_int {
        let capture = unsafe { &mut *context.cast::<Capture<'_>>() };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), EditingError> {
            if xyz.is_null() || capture.centers.len() as u64 >= capture.limits.max_outputs {
                return Err(EditingError::Budget("edge center output".into()));
            }
            let values = unsafe { std::slice::from_raw_parts(xyz, 3) };
            if values.iter().any(|value| !value.is_finite()) { return Err(EditingError::Invalid("edge center output".into())); }
            let live = (capture.centers.capacity() as u64).saturating_mul(std::mem::size_of::<Point3>() as u64);
            let pending = (capture.centers.len() as u64 + 1).checked_mul(2 * std::mem::size_of::<Point3>() as u64)
                .ok_or_else(|| EditingError::Budget("edge center allocation".into()))?;
            if live.checked_add(pending).is_none_or(|total| total > capture.limits.max_bytes) {
                return Err(EditingError::Budget("edge center allocation".into()));
            }
            (capture.admit)(live, pending).map_err(|error| EditingError::Budget(error.to_string()))?;
            capture.centers.try_reserve(1).map_err(|_| EditingError::Budget("edge center allocation".into()))?;
            let actual = (capture.centers.capacity() as u64).saturating_mul(std::mem::size_of::<Point3>() as u64);
            (capture.admit)(actual, 0).map_err(|error| EditingError::Budget(error.to_string()))?;
            capture.centers.push([values[0], values[1], values[2]]);
            Ok(())
        })).unwrap_or_else(|_| Err(EditingError::Invalid("edge center callback panic".into())));
        match result { Ok(()) => 0, Err(error) => { capture.error = Some(error); 1 } }
    }
    let mut capture = Capture { limits, centers: Vec::new(), error: None, admit: &mut admit };
    let mut message = [0 as c_char; 512];
    // SAFETY: callback context, source and all buffers remain valid until native work completes.
    let status = unsafe {
        let context = (&mut capture as *mut Capture<'_>).cast();
        let nodes = usize::try_from(limits.max_nodes).unwrap_or(usize::MAX);
        let outputs = usize::try_from(limits.max_outputs).unwrap_or(usize::MAX);
        if circular { om9_occt_circular_edge_centers(source.as_ptr(), source.len(), nodes, outputs,
            checkpoint, sink, context, message.as_mut_ptr(), 512) }
        else { om9_occt_source_vertices(source.as_ptr(), source.len(), nodes, outputs,
            checkpoint, sink, context, message.as_mut_ptr(), 512) }
    };
    if let Some(error) = capture.error { return Err(error); }
    if status != 0 {
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    Ok(capture.centers)
}

/// Extract edges belonging to exactly one source face without building a display mesh.
pub fn boundary_edges_from_source_admitted(source:&str, limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>,EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {
        return Err(EditingError::Invalid("edge source".into()));
    }
    edge_curves_raw_admitted(source,None,&[],None,None,&mut [0.;3],None,2,None,limits,&mut admit)
}

/// Parameter bounds of an unambiguous open single-face B-rep, in source coordinates.
pub fn single_face_isocurve_bounds(shape: &dyn BrepSourceAccess,limits:&Limits)->Result<[[f64;2];2],EditingError> {
    shape.validate_source(limits).map_err(|e|EditingError::Invalid(e.to_string()))?;
    single_face_isocurve_bounds_from_source(shape.source(),limits)
}

/// Inspect a caller-admitted OCCT source without constructing a display mesh.
pub fn single_face_isocurve_bounds_from_source(source:&str,limits:&Limits)->Result<[[f64;2];2],EditingError> {
    face_isocurve_bounds_from_source(source,None,limits)
}

#[derive(Clone,Copy)]
pub struct IsocurveFaceWitness {pub triangle:[Point3;3],pub deflection:f64,pub tolerance:f64}

/// Local-space view query. Its owner must bind it to the source and scene.
#[derive(Clone,Copy)]
pub struct IsocurveFaceRay {pub origin:Point3,pub direction:Point3,pub reference:Point3,pub tolerance:f64,pub max_distance:f64,pub witness:Option<IsocurveFaceWitness>}

#[derive(Clone,Copy)]
enum IsocurveFaceSelection { Witness(Option<IsocurveFaceWitness>), Ray(IsocurveFaceRay) }

pub fn face_isocurve_bounds_from_source(source:&str,witness:Option<IsocurveFaceWitness>,limits:&Limits)->Result<[[f64;2];2],EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {return Err(EditingError::Invalid("isocurve source".into()));}
    let mut bounds=[0.;4];let mut message=[0 as c_char;512];
    let status=unsafe {om9_occt_isocurve_face_bounds(source.as_ptr(),source.len(),
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),witness.as_ref().map_or(std::ptr::null(),|w|w.triangle.as_ptr().cast()),
        witness.map_or(0.,|w|w.deflection),witness.map_or(0.,|w|w.tolerance),bounds.as_mut_ptr(),message.as_mut_ptr(),message.len() as c_int)};
    if status!=0 {return Err(EditingError::Kernel(unsafe {std::ffi::CStr::from_ptr(message.as_ptr())}.to_string_lossy().into_owned()));}
    Ok([[bounds[0],bounds[1]],[bounds[2],bounds[3]]])
}

/// Inspect the exact face nearest `reference` on a local-space ray. The caller
/// admits source/native work and bounds `max_distance` for its viewport; this
/// returns no persistent face identity and does not authorize a later edit.
pub fn ray_face_isocurve_bounds_from_source(source:&str,origin:Point3,direction:Point3,reference:Point3,
    tolerance:f64,max_distance:f64,limits:&Limits)->Result<[[f64;2];2],EditingError> {
    ray_face_isocurve_hit_from_source(source,origin,direction,reference,tolerance,max_distance,limits).map(|(bounds,_)|bounds)
}

/// Returns the selected source face's bounds and exact local-space ray hit.
/// A viewport caller must check this hit in its own world/view context.
pub fn ray_face_isocurve_hit_from_source(source:&str,origin:Point3,direction:Point3,reference:Point3,
    tolerance:f64,max_distance:f64,limits:&Limits)->Result<([[f64;2];2],Point3),EditingError> {
    ray_face_isocurve_hit_with_witness_from_source(source,IsocurveFaceRay {origin,direction,reference,tolerance,max_distance,witness:None},limits)
}

pub fn ray_face_isocurve_hit_with_witness_from_source(source:&str,ray:IsocurveFaceRay,limits:&Limits)->Result<([[f64;2];2],Point3),EditingError> {
    try_ray_face_isocurve_hit_with_witness_from_source(source, ray, limits)?
        .ok_or_else(|| EditingError::Invalid("isocurve ray has no unambiguous matching face".into()))
}

/// Returns `None` only when there is no matching source face or ray hit.
/// Ambiguity, mismatches, kernel, admission and invalid-input failures remain errors.
pub fn try_ray_face_isocurve_hit_with_witness_from_source(source:&str,ray:IsocurveFaceRay,limits:&Limits)->Result<Option<([[f64;2];2],Point3)>,EditingError> {
    let IsocurveFaceRay {origin,direction,reference,tolerance,max_distance,witness}=ray;
    if source.is_empty() || source.len() as u64>limits.max_bytes ||
        !origin.iter().chain(direction.iter()).chain(reference.iter()).all(|v|v.is_finite()) ||
        !tolerance.is_finite() || tolerance<=0. || !max_distance.is_finite() || max_distance<=0. {
        return Err(EditingError::Invalid("isocurve ray input".into()));
    }
    if let Some(w)=witness {
        validate_face_triangles(&[w.triangle],w.tolerance)?;
        if !w.deflection.is_finite() || w.deflection<0. || !(w.deflection+w.tolerance).is_finite() {return Err(EditingError::Invalid("isocurve face witness".into()));}
    }
    let mut bounds=[0.;4];let mut resolved=[0.;3];let mut message=[0 as c_char;512];
    let status=unsafe {om9_occt_isocurve_ray_face_bounds(source.as_ptr(),source.len(),
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),origin.as_ptr(),direction.as_ptr(),reference.as_ptr(),
        tolerance,max_distance,witness.as_ref().map_or(std::ptr::null(),|w|w.triangle.as_ptr().cast()),
        witness.map_or(0.,|w|w.deflection),witness.map_or(0.,|w|w.tolerance),bounds.as_mut_ptr(),resolved.as_mut_ptr(),message.as_mut_ptr(),message.len() as c_int)};
    if status==3 {return Ok(None);}
    if status!=0 {return Err(EditingError::Kernel(unsafe {std::ffi::CStr::from_ptr(message.as_ptr())}.to_string_lossy().into_owned()));}
    if !resolved.iter().all(|v|v.is_finite()) {return Err(EditingError::Invalid("isocurve ray hit".into()));}
    Ok(Some(([[bounds[0],bounds[1]],[bounds[2],bounds[3]]],resolved)))
}

/// Clip an isoparametric curve against one source face, including its holes.
pub fn single_face_isocurve_admitted(shape: &dyn BrepSourceAccess,along_u:bool,value:f64,limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>)->Result<Vec<om9_core::geometry::Curve>,EditingError> {
    shape.validate_source(limits).map_err(|e|EditingError::Invalid(e.to_string()))?;
    single_face_isocurve_from_source_admitted(shape.source(),along_u,value,limits,&mut admit)
}

/// Extract from caller-admitted source while accounting for the caller's retained source bytes.
pub fn single_face_isocurve_from_source_admitted(source:&str,along_u:bool,value:f64,limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>)->Result<Vec<om9_core::geometry::Curve>,EditingError> {
    face_isocurve_from_source_admitted(source,along_u,value,None,limits,&mut admit)
}

/// Extract default-density interior wires from all faces in a single native
/// source read. Curves are clipped to each face, including inner boundaries;
/// they are display wires, not persistent topology edges. The caller owns and
/// admits the source for the call and must guard the publication context.
pub fn default_face_isocurves_from_source_admitted(source:&str,limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>)->Result<Vec<om9_core::geometry::Curve>,EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {
        return Err(EditingError::Invalid("isocurve source".into()));
    }
    edge_curves_raw_admitted(source,None,&[],None,None,&mut [0.;3],None,4,None,limits,&mut admit)
}

pub fn face_isocurve_from_source_admitted(source:&str,along_u:bool,value:f64,witness:Option<IsocurveFaceWitness>,limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>)->Result<Vec<om9_core::geometry::Curve>,EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {return Err(EditingError::Invalid("isocurve source".into()));}
    edge_curves_raw_admitted(source,None,&[],None,None,&mut [0.;3],None,3,Some((along_u,value,IsocurveFaceSelection::Witness(witness))),limits,&mut admit)
}

/// Construct exact isocurves from a tracked canonical lease, without retaining
/// a document or display. The caller must retain admission for the returned
/// curves, and must revalidate its command and scene before publication.
/// Cancellation is checked at admission callbacks and after the native call;
/// one in-flight native call cannot be interrupted.
pub fn face_isocurve_from_captured_source_cancellable(
    source: &om9_core::transaction::CapturedBrepSource, along_u: bool, value: f64,
    witness: Option<IsocurveFaceWitness>, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Vec<om9_core::geometry::Curve>> {
    isocurve_from_captured_source_cancellable(source, along_u, value,
        IsocurveFaceSelection::Witness(witness), limits, cancel, admit)
}

/// Resolve a face by its validated local-space ray against a captured source.
/// As with witness selection, callers retain output admission and revalidate
/// the selection before publishing; native work in flight is not interruptible.
pub fn ray_face_isocurve_from_captured_source_cancellable(
    source: &om9_core::transaction::CapturedBrepSource, along_u: bool, value: f64,
    ray: IsocurveFaceRay, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Vec<om9_core::geometry::Curve>> {
    isocurve_from_captured_source_cancellable(source, along_u, value,
        IsocurveFaceSelection::Ray(ray), limits, cancel, admit)
}

fn isocurve_from_captured_source_cancellable(
    source: &om9_core::transaction::CapturedBrepSource, along_u: bool, value: f64,
    selection: IsocurveFaceSelection, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Vec<om9_core::geometry::Curve>> {
    use om9_core::Error;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let canonical = source.validate_canonical_admitted(limits, &mut |live, pending| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        admit(live, pending)
    })?;
    let remaining = limits.remaining_after(canonical)?;
    let mut refusal = None;
    let mut admission = |live, pending| {
        let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
            canonical.bytes.checked_add(live).ok_or(Error::Overflow("captured isocurve admission"))
                .and_then(|held| admit(held, pending))
        };
        if let Err(error) = &result { refusal = Some(error.clone()); }
        result
    };
    let result = match selection {
        IsocurveFaceSelection::Witness(witness) => face_isocurve_from_source_admitted(
            source.source(), along_u, value, witness, &remaining, &mut admission),
        IsocurveFaceSelection::Ray(ray) => ray_face_isocurve_from_source_admitted(
            source.source(), along_u, value, ray, &remaining, &mut admission),
    };
    let curves = result.map_err(|error| refusal.unwrap_or_else(|| match error {
        EditingError::Budget(_) => Error::Budget("bytes"),
        other => Error::Io(format!("ExtractIsocurve: {other}")),
    }))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(curves)
}

/// Re-resolve a source face by the same local-space ray while extracting its
/// isocurve. Callers must also ensure the immutable source and selection have
/// not changed since requesting bounds; admission covers Rust outputs only.
pub fn ray_face_isocurve_from_source_admitted(source:&str,along_u:bool,value:f64,ray:IsocurveFaceRay,limits:&Limits,
    mut admit:impl FnMut(u64,u64)->om9_core::Result<()>)->Result<Vec<om9_core::geometry::Curve>,EditingError> {
    if source.is_empty() || source.len() as u64>limits.max_bytes {return Err(EditingError::Invalid("isocurve source".into()));}
    if let Some(w)=ray.witness {validate_face_triangles(&[w.triangle],w.tolerance)?; if !w.deflection.is_finite() || w.deflection<0. || !(w.deflection+w.tolerance).is_finite() {return Err(EditingError::Invalid("isocurve face witness".into()));}}
    edge_curves_raw_admitted(source,None,&[],None,None,&mut [0.;3],None,3,Some((along_u,value,IsocurveFaceSelection::Ray(ray))),limits,&mut admit)
}

/// Extract source edges used by exactly one face, from the same native topology map.
pub fn source_boundary_edges_nurbs_admitted(shape: &dyn BrepSourceAccess, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    edge_curves_admitted(shape, &[], None, None, &mut [0.; 3], None, 2, limits, &mut admit)
}

/// Resolve a local point to exactly one source edge and emit its curve from the
/// same native edge map. The caller must validate the point and tolerance against
/// the viewport and account all other live owners.
pub fn source_edge_curve_at_point_admitted(shape: &dyn BrepSourceAccess, point: Point3, tolerance: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<(om9_core::geometry::Curve, Point3), EditingError> {
    source_edge_curve_inner(shape, point, tolerance, None, false, limits, &mut admit)
}

/// Resolve a display segment only when exactly one source edge contains both endpoints.
pub fn source_edge_curve_at_segment_admitted(shape: &dyn BrepSourceAccess, point: Point3, tolerance: f64,
    segment: [Point3; 2], segment_tolerance: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<(om9_core::geometry::Curve, Point3), EditingError> {
    if !segment.iter().flatten().all(|v| v.is_finite()) || !segment_tolerance.is_finite() || segment_tolerance <= 0.0
        || (0..3).map(|i| (segment[0][i] - segment[1][i]).powi(2)).sum::<f64>() <= segment_tolerance.powi(2) {
        return Err(EditingError::Invalid("source edge pick segment".into()));
    }
    source_edge_curve_inner(shape, point, tolerance, Some((segment, segment_tolerance)), false, limits, &mut admit)
}

/// Resolve one edge from canonical source without constructing a display mesh.
/// The caller retains and admits the source string for the whole call.
pub fn source_edge_from_text_segment_admitted(source: &str, point: Point3, tolerance: f64,
    segment: [Point3; 2], segment_tolerance: f64, limits: &Limits,
    mut admit: impl FnMut(u64,u64)->om9_core::Result<()>) -> Result<(om9_core::geometry::Curve,Point3),EditingError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes
        || !point.iter().chain(segment.iter().flatten()).all(|v|v.is_finite())
        || !tolerance.is_finite() || tolerance <= 0.0 || !segment_tolerance.is_finite() || segment_tolerance <= 0.0
        || (0..3).map(|i|(segment[0][i]-segment[1][i]).powi(2)).sum::<f64>() <= segment_tolerance.powi(2) {
        return Err(EditingError::Invalid("source edge pick input".into()));
    }
    let mut resolved=[0.;3];
    let mut curves=edge_curves_raw_admitted(source,None,&[],Some((point,tolerance)),Some((segment,segment_tolerance)),
        &mut resolved,None,0,None,limits,&mut admit)?;
    if !resolved.iter().all(|v|v.is_finite()) { return Err(EditingError::Kernel("source edge point invalid".into())); }
    Ok((curves.pop().ok_or_else(||EditingError::Kernel("source edge output missing".into()))?,resolved))
}

/// As above, retaining the native circle's angular parameterization when applicable.
pub fn source_edge_analytic_at_segment_admitted(shape: &dyn BrepSourceAccess, point: Point3, tolerance: f64,
    segment: [Point3; 2], segment_tolerance: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<(om9_core::geometry::Curve, Point3), EditingError> {
    if !segment.iter().flatten().all(|v| v.is_finite()) || !segment_tolerance.is_finite() || segment_tolerance <= 0.0
        || (0..3).map(|i| (segment[0][i] - segment[1][i]).powi(2)).sum::<f64>() <= segment_tolerance.powi(2) {
        return Err(EditingError::Invalid("source edge pick segment".into()));
    }
    source_edge_curve_inner(shape, point, tolerance, Some((segment, segment_tolerance)), true, limits, &mut admit)
}

fn source_edge_curve_inner(shape: &dyn BrepSourceAccess, point: Point3, tolerance: f64, segment: Option<([Point3; 2], f64)>, analytic: bool, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> Result<(om9_core::geometry::Curve, Point3), EditingError> {
    if !point.iter().all(|value| value.is_finite()) || !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(EditingError::Invalid("source edge pick input".into()));
    }
    let mut resolved = [0.; 3];
    let mut circle = [0.; 14];
    let mut curves = edge_curves_admitted(shape, &[], Some((point, tolerance)), segment, &mut resolved,
        analytic.then_some(&mut circle), 0, limits, admit)?;
    if !resolved.iter().all(|value| value.is_finite()) { return Err(EditingError::Kernel("source edge point invalid".into())); }
    let curve = curves.pop().ok_or_else(|| EditingError::Kernel("source edge output missing".into()))?;
    let curve = if circle[0] == 1. { circle_from_native(&circle[1..], limits)? } else { curve };
    Ok((curve, resolved))
}

fn edge_curves_admitted(shape: &dyn BrepSourceAccess, indices: &[usize], point: Option<(Point3, f64)>, segment: Option<([Point3; 2], f64)>, resolved: &mut Point3, circle: Option<&mut [f64; 14]>, source_mode: u8, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>, EditingError> {
    shape.validate_source(limits).map_err(|error| EditingError::Invalid(error.to_string()))?;
    edge_curves_raw_admitted(shape.source(),Some(shape.topology().edges.len()),indices,point,segment,resolved,circle,source_mode,None,limits,admit)
}

fn edge_curves_raw_admitted(source:&str, edge_count:Option<usize>, indices:&[usize], point:Option<(Point3,f64)>, segment:Option<([Point3;2],f64)>, resolved:&mut Point3, circle:Option<&mut [f64;14]>, source_mode:u8, iso:Option<(bool,f64,IsocurveFaceSelection)>, limits:&Limits,
    admit:&mut dyn FnMut(u64,u64)->om9_core::Result<()>) -> Result<Vec<om9_core::geometry::Curve>,EditingError> {
    use om9_core::geometry::{Curve, NurbsCurve};
    let count = if point.is_some() { 1 } else { indices.len() };
    if (source_mode == 0 && count == 0) || count as u64 > limits.max_nodes || count as u64 > limits.max_outputs {
        return Err(EditingError::Budget("edge curve count".into()));
    }
    let base_bytes = count.checked_mul(std::mem::size_of::<i32>() + std::mem::size_of::<Curve>())
        .ok_or_else(|| EditingError::Budget("edge curve count".into()))?;
    if base_bytes as u64 > limits.max_bytes { return Err(EditingError::Budget("edge curve count".into())); }
    admit(0, base_bytes as u64).map_err(|e| EditingError::Budget(e.to_string()))?;
    let mut native_indices = Vec::new();
    std::alloc::Layout::array::<i32>(indices.len()).map_err(|_|EditingError::Budget("edge curve count".into()))?;
    native_indices.try_reserve_exact(indices.len()).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
    for &index in indices {
        if edge_count.is_none_or(|count|index>=count) { return Err(EditingError::Invalid("edge index out of range".into())); }
        native_indices.push(i32::try_from(index).map_err(|_| EditingError::Budget("edge index".into()))?);
    }
    struct Capture<'a> { limits: &'a Limits, result: Vec<Curve>, error: Option<EditingError>, bytes: usize, points: usize, expected: usize,
        admit: &'a mut dyn FnMut(u64, u64) -> om9_core::Result<()> }
    unsafe extern "C" fn checkpoint(context: *mut c_void) -> c_int {
        let capture = unsafe { &mut *context.cast::<Capture<'_>>() };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
            (capture.admit)(capture.bytes as u64, 0)))
            .unwrap_or_else(|_| Err(om9_core::Error::Invalid("isocurve checkpoint panic")));
        match result { Ok(()) => 0, Err(error) => { capture.error=Some(EditingError::Budget(error.to_string())); 1 } }
    }
    unsafe extern "C" fn prepare(context: *mut c_void, count: usize) -> c_int {
        let capture = unsafe { &mut *context.cast::<Capture<'_>>() };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), EditingError> {
            if count == 0 || count as u64 > capture.limits.max_nodes || count as u64 > capture.limits.max_outputs {
                return Err(EditingError::Budget("edge curve count".into()));
            }
            let bytes = count.checked_mul(std::mem::size_of::<Curve>()).ok_or_else(|| EditingError::Budget("edge curve count".into()))?;
            if bytes as u64 > capture.limits.max_bytes { return Err(EditingError::Budget("edge curve count".into())); }
            std::alloc::Layout::array::<Curve>(count).map_err(|_|EditingError::Budget("edge curve count".into()))?;
            (capture.admit)(capture.bytes as u64, bytes as u64).map_err(|e| EditingError::Budget(e.to_string()))?;
            capture.result.try_reserve_exact(count).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
            let actual = capture.result.capacity().checked_mul(std::mem::size_of::<Curve>()).ok_or_else(|| EditingError::Budget("edge curve count".into()))?;
            capture.bytes = capture.bytes.checked_add(actual).ok_or_else(|| EditingError::Budget("edge curve count".into()))?;
            if capture.bytes as u64 > capture.limits.max_bytes { return Err(EditingError::Budget("edge curve count".into())); }
            (capture.admit)(capture.bytes as u64, 0).map_err(|e| EditingError::Budget(e.to_string()))?;
            capture.expected = count;
            Ok(())
        })).unwrap_or_else(|_| Err(EditingError::Invalid("edge curve prepare panic".into())));
        match result { Ok(()) => 0, Err(error) => { capture.error = Some(error); 1 } }
    }
    unsafe extern "C" fn receive(context: *mut c_void, degree: c_int, points: *const f64,
        weights: *const f64, count: usize, knots: *const f64, knot_count: usize) -> c_int {
        let capture = unsafe { &mut *context.cast::<Capture<'_>>() };
        if capture.error.is_some() { return 1; }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<NurbsCurve, EditingError> {
            if capture.result.len() >= capture.expected { return Err(EditingError::Budget("edge curve count".into())); }
            let degree = usize::try_from(degree).map_err(|_| EditingError::Invalid("edge curve degree".into()))?;
            let bytes = count.checked_mul(4 * std::mem::size_of::<f64>())
                .and_then(|v| v.checked_add(knot_count.checked_mul(std::mem::size_of::<f64>())?))
                .ok_or_else(|| EditingError::Budget("edge curve size".into()))?;
            let peak = capture.bytes.checked_add(bytes.checked_mul(2).ok_or_else(|| EditingError::Budget("edge curve size".into()))?)
                .ok_or_else(|| EditingError::Budget("edge curve size".into()))?;
            let points_total = capture.points.checked_add(count).ok_or_else(|| EditingError::Budget("edge curve points".into()))?;
            if count == 0 || count as u64 > capture.limits.max_points || knot_count as u64 > capture.limits.max_nodes
                || points_total as u64 > capture.limits.max_points || degree == 0 || degree > capture.limits.max_degree || peak as u64 > capture.limits.max_bytes
                || points.is_null() || weights.is_null() || knots.is_null() {
                return Err(EditingError::Budget("edge curve output".into()));
            }
            std::alloc::Layout::array::<Point3>(count).map_err(|_|EditingError::Budget("edge curve size".into()))?;
            std::alloc::Layout::array::<f64>(count).map_err(|_|EditingError::Budget("edge curve size".into()))?;
            std::alloc::Layout::array::<f64>(knot_count).map_err(|_|EditingError::Budget("edge curve size".into()))?;
            let coords = unsafe { std::slice::from_raw_parts(points, count.checked_mul(3).ok_or_else(|| EditingError::Budget("edge curve points".into()))?) };
            let native_weights = unsafe { std::slice::from_raw_parts(weights, count) };
            let native_knots = unsafe { std::slice::from_raw_parts(knots, knot_count) };
            (capture.admit)(capture.bytes as u64, (bytes * 2) as u64)
                .map_err(|e| EditingError::Budget(e.to_string()))?;
            let mut poles = Vec::new(); let mut owned_weights = Vec::new(); let mut owned_knots = Vec::new();
            poles.try_reserve_exact(count).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
            owned_weights.try_reserve_exact(count).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
            owned_knots.try_reserve_exact(knot_count).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
            let actual = poles.capacity().checked_mul(std::mem::size_of::<Point3>())
                .and_then(|v| v.checked_add(owned_weights.capacity().checked_mul(std::mem::size_of::<f64>())?))
                .and_then(|v| v.checked_add(owned_knots.capacity().checked_mul(std::mem::size_of::<f64>())?))
                .ok_or_else(|| EditingError::Budget("edge curve capacity".into()))?;
            let actual_peak = capture.bytes.checked_add(actual).and_then(|v| v.checked_add(bytes))
                .ok_or_else(|| EditingError::Budget("edge curve capacity".into()))?;
            if actual_peak as u64 > capture.limits.max_bytes { return Err(EditingError::Budget("edge curve capacity".into())); }
            (capture.admit)((capture.bytes + actual) as u64, bytes as u64)
                .map_err(|e| EditingError::Budget(e.to_string()))?;
            for p in coords.chunks_exact(3) { poles.push([p[0], p[1], p[2]]); }
            owned_weights.extend_from_slice(native_weights); owned_knots.extend_from_slice(native_knots);
            let domain = [*owned_knots.get(degree).ok_or_else(|| EditingError::Invalid("edge curve knot domain".into()))?,
                *owned_knots.get(count).ok_or_else(|| EditingError::Invalid("edge curve knot domain".into()))?];
            let curve = NurbsCurve { degree, poles, weights: owned_weights, knots: owned_knots, domain,
                periodic: false, reversed: false, seam: None };
            curve.validate(capture.limits).map_err(|e| EditingError::Invalid(format!("edge curve: {e}")))?;
            capture.bytes = capture.bytes.checked_add(actual).ok_or_else(|| EditingError::Budget("edge curve size".into()))?;
            capture.points = points_total;
            Ok(curve)
        })).unwrap_or_else(|_| Err(EditingError::Invalid("edge curve callback panic".into())));
        match result { Ok(curve) => { capture.result.push(Curve::Nurbs(curve)); 0 }, Err(error) => { capture.error = Some(error); 1 } }
    }
    let mut curves = Vec::new();
    std::alloc::Layout::array::<Curve>(count).map_err(|_|EditingError::Budget("edge curve count".into()))?;
    curves.try_reserve_exact(count).map_err(|_| EditingError::Budget("edge curve allocation".into()))?;
    let actual_base = native_indices.capacity().checked_mul(std::mem::size_of::<i32>())
        .and_then(|v| v.checked_add(curves.capacity().checked_mul(std::mem::size_of::<Curve>())?))
        .ok_or_else(|| EditingError::Budget("edge curve count".into()))?;
    if actual_base as u64 > limits.max_bytes { return Err(EditingError::Budget("edge curve count".into())); }
    admit(actual_base as u64, 0).map_err(|e| EditingError::Budget(e.to_string()))?;
    let mut capture = Capture { limits, result: curves, error: None, bytes: actual_base, points: 0, expected: count, admit };
    let mut message = [0 as c_char; 512];
    let status = if let Some((point, tolerance)) = point {
        let segment_ptr = segment.as_ref().map_or(std::ptr::null(), |(points, _)| points.as_ptr().cast::<f64>());
        unsafe { om9_occt_point_edge_nurbs(source.as_ptr(), source.len(), point.as_ptr(), tolerance,
            segment_ptr, segment.as_ref().map_or(0.0, |(_, tolerance)| *tolerance), resolved.as_mut_ptr(),
            circle.map_or(std::ptr::null_mut(), |values| values.as_mut_ptr()),
            usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
            usize::try_from(limits.max_points).unwrap_or(usize::MAX), usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
            usize::try_from(limits.max_bytes / 2).unwrap_or(usize::MAX), i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
            receive, (&mut capture as *mut Capture<'_>).cast(), message.as_mut_ptr(), message.len() as c_int) }
    } else if let Some((along_u,value,IsocurveFaceSelection::Witness(witness)))=iso { unsafe {om9_occt_face_isocurve(source.as_ptr(),source.len(),i32::from(along_u),value,
        witness.as_ref().map_or(std::ptr::null(),|w|w.triangle.as_ptr().cast()),witness.map_or(0.,|w|w.deflection),witness.map_or(0.,|w|w.tolerance),
        usize::try_from(limits.max_points).unwrap_or(usize::MAX),usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        usize::try_from(limits.max_bytes/2).unwrap_or(usize::MAX),i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
        prepare,receive,(&mut capture as *mut Capture<'_>).cast(),message.as_mut_ptr(),message.len() as c_int)}
    } else if let Some((along_u,value,IsocurveFaceSelection::Ray(ray)))=iso { unsafe {om9_occt_ray_face_isocurve(source.as_ptr(),source.len(),i32::from(along_u),value,
        ray.origin.as_ptr(),ray.direction.as_ptr(),ray.reference.as_ptr(),ray.tolerance,ray.max_distance,
        ray.witness.as_ref().map_or(std::ptr::null(),|w|w.triangle.as_ptr().cast()),ray.witness.map_or(0.,|w|w.deflection),ray.witness.map_or(0.,|w|w.tolerance),
        usize::try_from(limits.max_points).unwrap_or(usize::MAX),usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        usize::try_from(limits.max_bytes/2).unwrap_or(usize::MAX),i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
        prepare,receive,(&mut capture as *mut Capture<'_>).cast(),message.as_mut_ptr(),message.len() as c_int)}
    } else if source_mode == 4 { unsafe { om9_occt_default_face_isocurves(source.as_ptr(),source.len(),
        usize::try_from(limits.max_points).unwrap_or(usize::MAX),usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        usize::try_from(limits.max_bytes/2).unwrap_or(usize::MAX),i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
        checkpoint,prepare,receive,(&mut capture as *mut Capture<'_>).cast(),message.as_mut_ptr(),message.len() as c_int)
    } } else if source_mode != 0 { unsafe { om9_occt_all_edges_nurbs(source.as_ptr(), source.len(),
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        match source_mode { 2 => 1, 5 => 2, _ => 0 },
        usize::try_from(limits.max_points).unwrap_or(usize::MAX), usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        usize::try_from(limits.max_bytes / 2).unwrap_or(usize::MAX), i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
        prepare, receive, (&mut capture as *mut Capture<'_>).cast(), message.as_mut_ptr(), message.len() as c_int) }
    } else { unsafe { om9_occt_edges_nurbs(source.as_ptr(), source.len(), native_indices.as_ptr(), native_indices.len(),
        usize::try_from(limits.max_points).unwrap_or(usize::MAX), usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        usize::try_from(limits.max_bytes / 2).unwrap_or(usize::MAX), i32::try_from(limits.max_degree).unwrap_or(i32::MAX),
        receive, (&mut capture as *mut Capture<'_>).cast(), message.as_mut_ptr(), message.len() as c_int) } };
    if let Some(error) = capture.error { return Err(error); }
    if status != 0 {
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    if capture.result.len() != capture.expected { return Err(EditingError::Kernel("edge curve output missing".into())); }
    Ok(capture.result)
}

fn checked_deflection(deflection_mm: f64) -> Result<f64, EditingError> {
    if !deflection_mm.is_finite() || deflection_mm <= 0.0 {
        return Err(EditingError::Invalid(
            "deflection must be finite and positive".into(),
        ));
    }
    Ok(deflection_mm)
}

fn checked_brep(brep: &dyn BrepSourceAccess, limits: &Limits) -> Result<Measures, EditingError> {
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
    a: &dyn BrepSourceAccess,
    b: &dyn BrepSourceAccess,
    op: BooleanOp,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    boolean_impl(a, b, op, deflection_mm, limits, false, false)
}

/// Reject results that fail OCCT's small-edge and self-interference checks before publication.
/// This conservative check can reject usable shapes; it does not prove Boolean correctness.
pub fn boolean_checked(
    a: &dyn BrepSourceAccess,
    b: &dyn BrepSourceAccess,
    op: BooleanOp,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    boolean_impl(a, b, op, deflection_mm, limits, true, false)
}

/// Checked Union may replace a strictly certified spline box with a temporary
/// equivalent analytic operand. Other Boolean callers retain their prior path.
pub fn boolean_checked_certified_union(
    a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    boolean_impl(a, b, BooleanOp::Union, deflection_mm, limits, true, true)
}

fn boolean_impl(
    a: &dyn BrepSourceAccess,
    b: &dyn BrepSourceAccess,
    op: BooleanOp,
    deflection_mm: f64,
    limits: &Limits,
    check_result: bool,
    certified_union: bool,
) -> Result<BooleanOutcome, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    // Boolean consumes only canonical sources. The native adapter checks the
    // source topology and solid membership while reading each shape; a stale or
    // missing presentation mesh must not determine the exact result.
    // Both legacy values continue to retain their display allocations while
    // OCCT runs. Count those live buffers even though they are not operands.
    let mut inventory_limits = *limits;
    inventory_limits.max_points = u64::MAX;
    inventory_limits.max_nodes = u64::MAX;
    let inventory = |shape: &dyn BrepSourceAccess| shape.storage_usage(&inventory_limits)
        .map_err(|error| EditingError::Budget(error.to_string()));
    let a_bytes = inventory(a)?.bytes;
    let b_bytes = if a.shares_validation_payload(b) { 0 } else { inventory(b)?.bytes };
    let held = a_bytes.checked_add(b_bytes).ok_or_else(|| EditingError::Budget("Boolean input accounting".into()))?;
    let remaining = limits.remaining_after(om9_core::budget::Usage { bytes: held, ..Default::default() })
        .map_err(|error| EditingError::Budget(error.to_string()))?;
    let source_limits = |source_bytes: u64| {
        let bytes = held.checked_sub(source_bytes)
            .ok_or_else(|| EditingError::Budget("Boolean source accounting".into()))?;
        limits.remaining_after(om9_core::budget::Usage { bytes, ..Default::default() })
            .map_err(|error| EditingError::Budget(error.to_string()))
    };
    let source_error = |error: om9_core::Error| match error {
        om9_core::Error::Budget(_) | om9_core::Error::Overflow(_) => EditingError::Budget(error.to_string()),
        _ => EditingError::Invalid(error.to_string()),
    };
    a.validate_source(&source_limits(a.source_storage_bytes().map_err(|error| EditingError::Budget(error.to_string()))?)?)
        .map_err(source_error)?;
    if b_bytes != 0 {
        b.validate_source(&source_limits(b.source_storage_bytes().map_err(|error| EditingError::Budget(error.to_string()))?)?)
            .map_err(source_error)?;
    }
    boolean_sources_impl(a.source(),b.source(),op,deflection,&remaining,check_result,certified_union,None)
}

fn boolean_sources_impl(a:&str,b:&str,op:BooleanOp,deflection:f64,limits:&Limits,check_result:bool,certified_union:bool,
    admission:Option<&mut dyn FnMut(u64,u64)->om9_core::Result<()>>)->Result<BooleanOutcome,EditingError> {
    let budget = snapshot_budget(limits);
    let invoke=|sink, context, message, capacity| {
        // SAFETY: both sources, the callback context and every output live through this call.
        unsafe {
            om9_occt_boolean(
                op.native(),
                if certified_union { 3 } else if check_result { 2 } else { 1 },
                a.as_ptr(),
                a.len(),
                b.as_ptr(),
                b.len(),
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
    };
    let brep = match admission {
        Some(admit)=>crate::brep::snapshot_via_optional_with_admission(deflection,limits,admit,invoke),
        None=>snapshot_via_optional(deflection,limits,invoke),
    }.map_err(EditingError::from)?;
    Ok(match brep {
        Some(brep) => BooleanOutcome::Body(brep),
        None => BooleanOutcome::Empty,
    })
}

/// Checked Boolean from independently tracked source leases. The caller must
/// retain the admitted output allocation and revalidate its transaction before
/// publication. Native work in progress cannot be interrupted; Rust snapshot
/// copying observes cancellation and the supplied admission callback.
pub fn boolean_captured_sources_cancellable(
    a:&om9_core::transaction::CapturedBrepSource,b:&om9_core::transaction::CapturedBrepSource,
    op:BooleanOp,deflection_mm:f64,certified_union:bool,limits:&Limits,
    cancel:&om9_core::transaction::Cancellation,mut admit:impl FnMut(u64,u64)->om9_core::Result<()>,
)->om9_core::Result<BooleanOutcome> {
    use om9_core::Error;
    if cancel.is_canceled(){return Err(Error::Canceled);}
    if certified_union && op!=BooleanOp::Union {return Err(Error::Invalid("certified Boolean operation"));}
    let deflection=checked_deflection(deflection_mm).map_err(|error|Error::Io(error.to_string()))?;
    let first=a.validate_canonical_admitted(limits,|live,pending|{
        if cancel.is_canceled(){return Err(Error::Canceled);}admit(live,pending)
    })?;
    let second=if a.same_canonical_allocations(b) {om9_core::budget::Usage::default()} else {
        b.validate_canonical_admitted(limits,|live,pending|{
            if cancel.is_canceled(){return Err(Error::Canceled);}
            admit(first.bytes.checked_add(live).ok_or(Error::Overflow("Boolean source admission"))?,pending)
        })?
    };
    let mut held=first;held.add(second,limits)?;let remaining=limits.remaining_after(held)?;
    let mut refusal=None;
    let result=boolean_sources_impl(a.source(),b.source(),op,deflection,&remaining,true,certified_union,Some(&mut |live,pending| {
        let result=if cancel.is_canceled(){Err(Error::Canceled)} else {
            held.bytes.checked_add(live).ok_or(Error::Overflow("Boolean output admission"))
                .and_then(|total|admit(total,pending))
        };
        if let Err(error)=&result {refusal=Some(error.clone());}result
    }));
    let result=result.map_err(|error|refusal.unwrap_or_else(||match error {
        EditingError::Budget(_)=>Error::Budget("bytes"),other=>Error::Io(format!("Boolean: {other}")),
    }))?;
    if cancel.is_canceled(){return Err(Error::Canceled);}
    let output = match &result {
        BooleanOutcome::Body(body) => body.storage_usage(&remaining)?,
        BooleanOutcome::Empty => om9_core::budget::Usage::default(),
    };
    held.add(output, limits)?;
    admit(held.bytes, 0)?;
    if cancel.is_canceled(){return Err(Error::Canceled);}
    Ok(result)
}

/// Checked Boolean on captured local sources with explicit local-to-operation
/// transforms. Intermediate transformed bodies are private and accounted;
/// callers must retain the result reservation and revalidate before commit.
pub fn boolean_placed_captured_sources_cancellable(
    a: &om9_core::transaction::CapturedBrepSource,
    a_transform: &om9_core::geometry::Transform,
    b: &om9_core::transaction::CapturedBrepSource,
    b_transform: &om9_core::geometry::Transform,
    op: BooleanOp,
    deflection_mm: f64,
    certified_union: bool,
    limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<BooleanOutcome> {
    use om9_core::{Error, budget::Usage, geometry::Transform};
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if certified_union && op != BooleanOp::Union { return Err(Error::Invalid("certified Boolean operation")); }
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let first = a.validate_canonical_admitted(limits, |live, pending| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        admit(live, pending)
    })?;
    let second = if a.same_canonical_allocations(b) { Usage::default() } else {
        b.validate_canonical_admitted(&limits.remaining_after(first)?, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(first.bytes.checked_add(live).ok_or(Error::Overflow("Boolean source admission"))?, pending)
        })?
    };
    let mut held = first;
    held.add(second, limits)?;
    let mut transform = |source: &om9_core::transaction::CapturedBrepSource,
                         matrix: &Transform, source_usage: Usage, held: &mut Usage| -> om9_core::Result<Option<Brep>> {
        if *matrix == Transform::identity() { return Ok(None); }
        let other = held.bytes.checked_sub(source_usage.bytes).ok_or(Error::Overflow("Boolean placement admission"))?;
        let reduced = limits.remaining_after(Usage { bytes: other, ..Usage::default() })?;
        let body = crate::transform_ops::transform_captured_source_cancellable(source, matrix, deflection,
            &reduced, cancel, |live, pending| {
                admit(other.checked_add(live).ok_or(Error::Overflow("Boolean placement admission"))?, pending)
            })?;
        held.add(body.storage_usage(&limits.remaining_after(*held)?)?, limits)?;
        Ok(Some(body))
    };
    let transformed_a = transform(a, a_transform, first, &mut held)?;
    // For shared source allocations both transforms still refer to the first charge.
    let transformed_b = transform(b, b_transform, if second.bytes == 0 { first } else { second }, &mut held)?;
    drop(transform);
    let a_source = transformed_a.as_ref().map_or_else(|| a.source(), |body| body.source());
    let b_source = transformed_b.as_ref().map_or_else(|| b.source(), |body| body.source());
    let remaining = limits.remaining_after(held)?;
    let mut refusal = None;
    let outcome = boolean_sources_impl(a_source, b_source, op, deflection, &remaining, true, certified_union,
        Some(&mut |live, pending| {
            let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
                held.bytes.checked_add(live).ok_or(Error::Overflow("Boolean output admission"))
                    .and_then(|total| admit(total, pending))
            };
            if let Err(error) = &result { refusal = Some(error.clone()); }
            result
        })).map_err(|error| refusal.unwrap_or_else(|| match error {
            EditingError::Budget(_) => Error::Budget("bytes"),
            other => Error::Io(format!("Boolean: {other}")),
        }))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if let BooleanOutcome::Body(body) = &outcome { held.add(body.storage_usage(&remaining)?, limits)?; }
    admit(held.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(outcome)
}

/// Symmetric difference for a Boolean Two Objects preview. Both source leases,
/// each difference and the final union are admitted until no longer live.
pub fn boolean_xor_placed_captured_sources_cancellable(
    a: &om9_core::transaction::CapturedBrepSource, a_transform: &om9_core::geometry::Transform,
    b: &om9_core::transaction::CapturedBrepSource, b_transform: &om9_core::geometry::Transform,
    deflection: f64, limits: &Limits, cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<BooleanOutcome> {
    use om9_core::{Error, budget::Usage};
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let other_bytes = if a.same_canonical_allocations(b) { 0 } else { b.canonical_storage_bytes()? };
    let first_limits = limits.remaining_after(Usage { bytes: other_bytes, ..Usage::default() })?;
    let first = a.validate_canonical_admitted(&first_limits, |live, pending| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        admit(other_bytes.checked_add(live).ok_or(Error::Overflow("Boolean XOR sources"))?, pending)
    })?;
    let second = if a.same_canonical_allocations(b) { Usage::default() } else {
        b.validate_canonical_admitted(&limits.remaining_after(first)?, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(first.bytes.checked_add(live).ok_or(Error::Overflow("Boolean XOR sources"))?, pending)
        })?
    };
    let mut source_usage = first;
    source_usage.add(second, limits)?;
    let left = boolean_placed_captured_sources_cancellable(a, a_transform, b, b_transform,
        BooleanOp::Difference, deflection, false, limits, cancel, |live, pending| admit(live.max(source_usage.bytes), pending))?;
    let left_usage = match &left {
        BooleanOutcome::Body(body) => body.storage_usage(&limits.remaining_after(source_usage)?)?,
        BooleanOutcome::Empty => Usage::default(),
    };
    let remaining = limits.remaining_after(left_usage)?;
    let right = boolean_placed_captured_sources_cancellable(b, b_transform, a, a_transform,
        BooleanOp::Difference, deflection, false, &remaining, cancel, |live, pending| {
            admit(left_usage.bytes.checked_add(live.max(source_usage.bytes)).ok_or(Error::Overflow("Boolean XOR admission"))?, pending)
        })?;
    let result = match (left, right) {
        (BooleanOutcome::Empty, BooleanOutcome::Empty) => BooleanOutcome::Empty,
        (BooleanOutcome::Body(body), BooleanOutcome::Empty) | (BooleanOutcome::Empty, BooleanOutcome::Body(body)) => BooleanOutcome::Body(body),
        (BooleanOutcome::Body(left), BooleanOutcome::Body(right)) => {
            let reduced = limits.remaining_after(source_usage)?;
            boolean_breps_cancellable_admitted(&left, &right, BooleanOp::Union, deflection, true, &reduced, cancel,
                &mut |live, pending| admit(source_usage.bytes.checked_add(live).ok_or(Error::Overflow("Boolean XOR union"))?, pending))?
        }
    };
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let mut retained = source_usage;
    if let BooleanOutcome::Body(body) = &result { retained.add(body.storage_usage(&limits.remaining_after(retained)?)?, limits)?; }
    admit(retained.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(result)
}

/// Fold a certified union over captured sources and their operation-frame
/// placements. The caller retains every source and the admitted output until
/// it has revalidated and published the result.
pub fn boolean_union_captured_group_cancellable(
    inputs: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    deflection_mm: f64,
    limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Brep> {
    if inputs.len() < 2 { return Err(om9_core::Error::Missing("Boolean union inputs")); }
    boolean_union_captured_group_impl(inputs, deflection_mm, limits, cancel, true, admit)
}

fn boolean_union_captured_group_impl(
    inputs: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    deflection_mm: f64, limits: &Limits, cancel: &om9_core::transaction::Cancellation,
    certified: bool, mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Brep> {
    let source = boolean_union_captured_source_impl(inputs, limits, cancel, certified, &mut admit)?;
    let outside = om9_core::budget::Usage { bytes: source.capacity() as u64, ..Default::default() };
    let remaining = limits.remaining_after(outside)?;
    let captured = crate::brep::capture_source_topology(&source, deflection_mm, 0.3, &remaining, |live, pending| {
        if cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
        admit(outside.bytes.checked_add(live).ok_or(om9_core::Error::Overflow("Union final display"))?, pending)
    }).map_err(|e| om9_core::Error::Io(e.to_string()))?;
    if cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
    Ok(captured.into_parts().0)
}

/// Canonical-only Union fold: no placement or intermediate tessellation. The
/// returned source stays charged until the caller captures its final display.
pub fn boolean_union_captured_source_cancellable(
    inputs: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    limits: &Limits, cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<String> {
    if inputs.len() < 2 { return Err(om9_core::Error::Missing("Boolean union inputs")); }
    boolean_union_captured_source_impl(inputs, limits, cancel, true, &mut admit)
}

fn boolean_union_captured_source_impl(
    inputs: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    limits: &Limits, cancel: &om9_core::transaction::Cancellation, certified: bool,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<String> {
    use om9_core::{Error, budget::Usage, geometry::Transform};
    if inputs.is_empty() { return Err(Error::Missing("Boolean union inputs")); }
    let mut held = Usage::default();
    for (source, _) in inputs {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let remaining = limits.remaining_after(held)?;
        let usage = source.validate_canonical_admitted(&remaining, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(held.bytes.checked_add(live).ok_or(Error::Overflow("Union inputs"))?, pending)
        })?;
        held.add(usage, limits)?;
    }
    let source_bytes = held.bytes;
    let mut current = String::new();
    for (index, (source, placement)) in inputs.iter().enumerate() {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        placement.validate()?;
        let outside = source_bytes.checked_add(current.capacity() as u64).ok_or(Error::Overflow("Union sources"))?;
        let transformed = if *placement != Transform::identity() || inputs.len() == 1 {
            let remaining = limits.remaining_after(Usage { bytes: outside, ..held })?;
            let budget = snapshot_budget(&remaining);
            Some(crate::brep::intermediate::source_via(&remaining, &mut |live, pending| {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                admit(outside.checked_add(live).ok_or(Error::Overflow("Union placement"))?, pending)
            }, |sink, context, message, capacity| {
                // SAFETY: the immutable source, matrix and synchronous sink live through the call.
                unsafe { om9_occt_transform_source(source.source().as_ptr(), source.source().len(),
                    placement.0.as_ptr().cast(), budget.nodes, budget.bytes, sink, context, message, capacity) }
            })?)
        } else { None };
        let right = transformed.as_deref().unwrap_or_else(|| source.source());
        if index == 0 {
            if let Some(owned) = transformed { current = owned; }
            continue;
        }
        let outside = outside.checked_add(transformed.as_ref().map_or(0, |s| s.capacity() as u64))
            .ok_or(Error::Overflow("Union operand"))?;
        let remaining = limits.remaining_after(Usage { bytes: outside, ..held })?;
        let budget = snapshot_budget(&remaining);
        let left = if current.is_empty() { inputs[0].0.source() } else { &current };
        let next = crate::brep::intermediate::source_via(&remaining, &mut |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(outside.checked_add(live).ok_or(Error::Overflow("Union output"))?, pending)
        }, |sink, context, message, capacity| {
            // SAFETY: both sources and callback remain borrowed until the adapter returns.
            unsafe { om9_occt_boolean_source(i32::from(certified), left.as_ptr(), left.len(), right.as_ptr(), right.len(),
                budget.nodes, budget.bytes, sink, context, message, capacity) }
        })?;
        current = next;
        drop(transformed);
        admit(source_bytes.checked_add(current.capacity() as u64).ok_or(Error::Overflow("Union retained source"))?, 0)?;
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(current)
}

/// Union each captured set using the checked group semantics, then perform a
/// checked difference or intersection. Source leases and intermediate bodies
/// remain charged until the returned output is transferred by the caller.
pub fn boolean_binary_captured_groups_cancellable(
    targets: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    cutters: &[(om9_core::transaction::CapturedBrepSource, om9_core::geometry::Transform)],
    op: BooleanOp, deflection_mm: f64, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<BooleanOutcome> {
    use om9_core::{Error, budget::Usage};
    if targets.is_empty() || cutters.is_empty() { return Err(Error::Missing("Boolean sets")); }
    if !matches!(op, BooleanOp::Difference | BooleanOp::Intersection) { return Err(Error::Invalid("Boolean binary operation")); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let mut held = Usage::default();
    let mut target_sources = Usage::default();
    let mut cutter_sources = Usage::default();
    for (group, total) in [(targets, &mut target_sources), (cutters, &mut cutter_sources)] {
        for (source, _) in group {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let remaining = limits.remaining_after(held)?;
            let usage = source.validate_canonical_admitted(&remaining, |live, pending| {
                if cancel.is_canceled() { return Err(Error::Canceled); }
                admit(held.bytes.checked_add(live).ok_or(Error::Overflow("Boolean sets admission"))?, pending)
            })?;
            held.add(usage, limits)?;
            total.add(usage, limits)?;
        }
    }
    let target_limits = limits.remaining_after(cutter_sources)?;
    let target = boolean_union_captured_group_impl(targets, deflection, &target_limits, cancel, false, |live, pending| {
        admit(cutter_sources.bytes.checked_add(live).ok_or(Error::Overflow("Boolean target admission"))?, pending)
    })?;
    let target_usage = target.storage_usage(&limits.remaining_after(held)?)?;
    held.add(target_usage, limits)?;
    let mut offset = target_sources;
    offset.add(target_usage, limits)?;
    let cutter_limits = limits.remaining_after(offset)?;
    let cutter = boolean_union_captured_group_impl(cutters, deflection, &cutter_limits, cancel, false, |live, pending| {
        admit(offset.bytes.checked_add(live).ok_or(Error::Overflow("Boolean cutter admission"))?, pending)
    })?;
    held.add(cutter.storage_usage(&limits.remaining_after(held)?)?, limits)?;
    let remaining = limits.remaining_after(held)?;
    let mut refusal = None;
    let outcome = boolean_sources_impl(target.source(), cutter.source(), op, deflection, &remaining, true, false,
        Some(&mut |live, pending| {
            let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
                held.bytes.checked_add(live).ok_or(Error::Overflow("Boolean output admission"))
                    .and_then(|total| admit(total, pending))
            };
            if let Err(error) = &result { refusal = Some(error.clone()); }
            result
        })).map_err(|error| refusal.unwrap_or_else(|| match error {
            EditingError::Budget(_) => Error::Budget("bytes"),
            other => Error::Io(format!("Boolean: {other}")),
        }))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if let BooleanOutcome::Body(body) = &outcome { held.add(body.storage_usage(&remaining)?, limits)?; }
    admit(held.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(outcome)
}

/// Subtract several provably disjoint solid tools in one operation. `None` requests the caller's
/// general overlapping-tool path; `Some(Empty)` is a successful cut with no remaining faces.
pub fn cut_disjoint_tools(
    target: &dyn BrepSourceAccess, tools: &[Brep], deflection_mm: f64, limits: &Limits,
) -> Result<Option<BooleanOutcome>, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    if tools.is_empty() { return Err(EditingError::Invalid("boolean requires tools".into())); }
    // Pairwise bounding-box screening is quadratic; keep large groups on the general path.
    if tools.len() > 128 { return Ok(None); }
    target.validate(limits).map_err(|e| EditingError::Invalid(e.to_string()))?;
    let mut pointers = Vec::new();
    let mut lengths = Vec::new();
    pointers.try_reserve_exact(tools.len()).map_err(|_| EditingError::Budget("boolean tools".into()))?;
    lengths.try_reserve_exact(tools.len()).map_err(|_| EditingError::Budget("boolean tools".into()))?;
    for tool in tools {
        tool.validate(limits).map_err(|e| EditingError::Invalid(e.to_string()))?;
        pointers.push(tool.source().as_ptr());
        lengths.push(tool.source().len());
    }
    let budget = snapshot_budget(limits);
    let mut fallback = false;
    let result = snapshot_via_optional(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: all sources, pointer arrays and callback context live through this synchronous call.
        let status = unsafe { om9_occt_cut_disjoint_tools(target.source().as_ptr(), target.source().len(),
            pointers.as_ptr(), lengths.as_ptr(), pointers.len(), deflection, budget.points,
            budget.nodes, budget.bytes, sink, context, message, capacity) };
        if status == 4 { fallback = true; 3 } else { status }
    }).map_err(EditingError::from)?;
    if fallback { return Ok(None); }
    Ok(Some(match result { Some(body) => BooleanOutcome::Body(body), None => BooleanOutcome::Empty }))
}

/// Cut a surface-only B-Rep by a solid tool, preserving exact face geometry.
/// A disjoint tool returns the unchanged surface; complete removal returns Empty.
/// The caller owns classification, publication and any subsequent joining.
pub fn trim_surface_by_solid(
    surface: &dyn BrepSourceAccess,
    tool: &dyn BrepSourceAccess,
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
        unsafe { om9_occt_boolean(BooleanOp::Difference.native(), 0, surface.source().as_ptr(), surface.source().len(),
            tool.source().as_ptr(), tool.source().len(), deflection, budget.points, budget.nodes,
            budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)?;
    Ok(match result { Some(brep) => BooleanOutcome::Body(brep), None => BooleanOutcome::Empty })
}

/// Intersect an open surface/shell with a closed solid without changing either input.
/// Empty intersections are reported explicitly; classification belongs to the caller.
pub fn intersect_surface_with_solid(
    surface: &dyn BrepSourceAccess, solid: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits,
) -> Result<BooleanOutcome, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let source = checked_brep(surface, limits)?;
    let cutter = checked_brep(solid, limits)?;
    if source.faces == 0 || source.solids != 0 { return Err(EditingError::Invalid("surface intersection requires surface-only input".into())); }
    if cutter.solids == 0 || !cutter.closed { return Err(EditingError::Invalid("surface intersection requires a closed solid".into())); }
    let budget = snapshot_budget(limits);
    let result = snapshot_via_optional(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: validated sources and callback context remain alive during the synchronous call.
        unsafe { om9_occt_boolean(BooleanOp::Intersection.native(), 0, surface.source().as_ptr(), surface.source().len(),
            solid.source().as_ptr(), solid.source().len(), deflection, budget.points, budget.nodes,
            budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)?;
    Ok(match result { Some(brep) => BooleanOutcome::Body(brep), None => BooleanOutcome::Empty })
}

/// Construct a finite solid carrier of a single cylindrical face and return its
/// axis origin, unit direction and radius; neither native handles nor source ownership escape.
pub fn boss_cylinder_carrier(boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits) -> Result<(Brep, [f64;7]), EditingError> {
    boss_cylinder_carrier_extended(boundary, 0.0, deflection_mm, limits)
}

fn boss_cylinder_carrier_extended(boundary: &dyn BrepSourceAccess, extension: f64, deflection_mm: f64, limits: &Limits) -> Result<(Brep, [f64;7]), EditingError> {
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
pub fn boss_cylindrical_polygon(profile: &crate::creation::Profile, boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    boss_cylindrical_polygons(&[profile], boundary, deflection_mm, limits)
}

/// Build disjoint polygon Boss walls on one cylindrical face as one result.
pub fn boss_cylindrical_polygons(profiles: &[&crate::creation::Profile], boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    boss_cylindrical_impl(profiles.iter().map(|p| CylindricalSection::Polygon(p)),profiles.len(),boundary,deflection_mm,limits)
}

/// Build disjoint exact polygon or curve Boss walls on one cylindrical face.
pub fn boss_cylindrical_sections(profiles: &[BossSection], boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
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

fn boss_cylindrical_impl<'a>(profiles: impl Iterator<Item=CylindricalSection<'a>>, count: usize, boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
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
    let mut cut = crate::transform_ops::transform_brep(boundary, &om9_core::geometry::Transform::identity(), deflection_mm, limits)?;
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
    surface: &dyn BrepSourceAccess, origin: Point3, normal: Point3, remove_point: Point3,
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
    boundary: &dyn BrepSourceAccess,
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
    distance: f64, boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    rib_straight_planar_boundary_many(&[(points, normal)], distance, boundary, deflection_mm, limits)
}

/// Development-only multi-curve version. All strips must be disjoint and
/// individually valid; construction fails without changing any input.
pub fn rib_straight_planar_boundary_many(
    curves: &[(&[om9_core::geometry::Point3], om9_core::geometry::Point3)],
    distance: f64, boundary: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits,
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
    boundary: &dyn BrepSourceAccess,
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

fn planar_faces_plane(boundary: &dyn BrepSourceAccess, allow_multiple: bool, limits: &Limits) -> Result<[f64; 6], EditingError> {
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
    boundary: &dyn BrepSourceAccess,
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
    let mut cut = crate::transform_ops::transform_brep(boundary, &om9_core::geometry::Transform::identity(), deflection_mm, limits)?;
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

/// Partition two source leases without retaining their document displays.
/// The caller keeps the admitted compound reservation alive until it has
/// extracted or published the pieces. Native calls cannot be interrupted.
pub fn split_captured_sources_cancellable(
    target: &om9_core::transaction::CapturedBrepSource,
    cutter: &om9_core::transaction::CapturedBrepSource,
    deflection_mm: f64,
    limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<SplitResult> {
    use om9_core::{Error, budget::Usage};
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let first = target.validate_canonical_admitted(limits, |live, pending| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        admit(live, pending)
    })?;
    let second = if target.same_canonical_allocations(cutter) { Usage::default() } else {
        cutter.validate_canonical_admitted(limits, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(first.bytes.checked_add(live).ok_or(Error::Overflow("split source admission"))?, pending)
        })?
    };
    let mut held = first;
    held.add(second, limits)?;
    split_validated_sources(target.source(), cutter.source(), deflection, limits, held, cancel, &mut admit)
}

fn split_validated_sources(
    target: &str, cutter: &str, deflection: f64, limits: &Limits,
    mut held: om9_core::budget::Usage, cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<SplitResult> {
    use om9_core::Error;
    let remaining = limits.remaining_after(held)?;
    let original = crate::creation::measure_validated_source(target).map_err(|e| Error::Io(format!("split target: {e}")))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let tool = crate::creation::measure_validated_source(cutter).map_err(|e| Error::Io(format!("split cutter: {e}")))?;
    if original.solids != 1 || tool.solids != 1 || !original.closed || !tool.closed {
        return Err(Error::Invalid("split needs two closed single solids"));
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let budget = snapshot_budget(&remaining);
    let mut pieces = 0;
    let mut refusal = None;
    let brep = crate::brep::snapshot_via_with_admission(deflection, &remaining, Some(&mut |live, pending| {
        let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
            held.bytes.checked_add(live).ok_or(Error::Overflow("split output admission"))
                .and_then(|total| admit(total, pending))
        };
        if let Err(error) = &result { refusal = Some(error.clone()); }
        result
    }), |sink, context, message, capacity| {
        // SAFETY: both source buffers, scalar output and callback remain live.
        unsafe { om9_occt_split_solid(target.as_ptr(), target.len(),
            cutter.as_ptr(), cutter.len(), &mut pieces, deflection,
            budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(|error| refusal.unwrap_or_else(|| Error::Io(format!("split: {error}"))))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let count = usize::try_from(pieces).unwrap_or(0);
    let result = crate::creation::measure_validated_source(brep.source()).map_err(|e| Error::Io(format!("split result: {e}")))?;
    if count == 0 || result.solids != count || !result.closed
        || (result.volume_mm3 - original.volume_mm3).abs() > original.volume_mm3.abs() * 1e-6 + 1e-9 {
        return Err(Error::Invalid("split solid failed topology or volume validation"));
    }
    held.add(brep.storage_usage(&remaining)?, limits)?;
    admit(held.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(SplitResult { brep, pieces: count })
}

/// Split intermediate B-reps while charging both live input representations.
/// Shared complete allocations are counted once; partial sharing is conservative.
pub fn split_breps_cancellable_admitted(
    target: &dyn BrepSourceAccess, cutter: &dyn BrepSourceAccess, deflection_mm: f64, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<SplitResult> {
    use om9_core::Error;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let held = validate_brep_pair_admitted(target, cutter, limits, cancel, &mut admit)?;
    split_validated_sources(target.source(), cutter.source(), deflection, limits, held, cancel, &mut admit)
}

fn validate_brep_pair_admitted(
    target: &dyn BrepSourceAccess, cutter: &dyn BrepSourceAccess, limits: &Limits, cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<om9_core::budget::Usage> {
    use om9_core::{Error, budget::Usage};
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let mut held = target.storage_usage(limits)?;
    let shared = target.shares_validation_payload(cutter);
    if !shared { held.add(cutter.storage_usage(limits)?, limits)?; }
    limits.remaining_after(held)?;
    let mut validate = |body: &dyn BrepSourceAccess| -> om9_core::Result<()> {
        let source_bytes = body.source_storage_bytes()?;
        let other = held.bytes.checked_sub(source_bytes).ok_or(Error::Overflow("split source accounting"))?;
        let remaining = limits.remaining_after(Usage { bytes: other, ..Usage::default() })?;
        body.validate_source_admitted(&remaining, &mut |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(other.checked_add(live).ok_or(Error::Overflow("split source admission"))?, pending)
        })?;
        Ok(())
    };
    validate(target)?;
    if !shared { validate(cutter)?; }
    Ok(held)
}

fn boolean_breps_cancellable_admitted(
    a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, op: BooleanOp, deflection: f64, checked: bool, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<BooleanOutcome> {
    use om9_core::Error;
    let mut held = validate_brep_pair_admitted(a, b, limits, cancel, admit)?;
    let remaining = limits.remaining_after(held)?;
    let mut refusal = None;
    let result = boolean_sources_impl(a.source(), b.source(), op, deflection, &remaining, checked, false,
        Some(&mut |live, pending| {
            let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
                held.bytes.checked_add(live).ok_or(Error::Overflow("split duplicate admission"))
                    .and_then(|total| admit(total, pending))
            };
            if let Err(error) = &result { refusal = Some(error.clone()); }
            result
        })).map_err(|error| refusal.unwrap_or_else(|| match error {
            EditingError::Budget(_) => Error::Budget("bytes"),
            other => Error::Io(format!("split duplicate: {other}")),
        }))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if let BooleanOutcome::Body(body) = &result { held.add(body.storage_usage(&remaining)?, limits)?; }
    admit(held.bytes, 0)?;
    Ok(result)
}

/// Extract one split-local piece while accounting the retained compound and
/// copied output. The caller accounts any other simultaneously retained pieces.
pub fn split_piece_cancellable_admitted(
    split: &SplitResult, index: usize, deflection_mm: f64, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Brep> {
    use om9_core::{Error, budget::Usage};
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if index >= split.pieces { return Err(Error::Invalid("split solid piece index")); }
    let index = c_int::try_from(index).map_err(|_| Error::Overflow("split piece index"))?;
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let held = split.brep.storage_usage(limits)?;
    let source_bytes = split.brep.source_storage_bytes()?;
    let display_bytes = held.bytes.checked_sub(source_bytes).ok_or(Error::Overflow("split compound accounting"))?;
    let source_limits = limits.remaining_after(Usage { bytes: display_bytes, ..Usage::default() })?;
    split.brep.validate_source_admitted(&source_limits, |live, pending| {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        admit(display_bytes.checked_add(live).ok_or(Error::Overflow("split compound admission"))?, pending)
    })?;
    let remaining = limits.remaining_after(held)?;
    let measured = crate::creation::measure_validated_source(split.brep.source())
        .map_err(|e| Error::Io(format!("split compound: {e}")))?;
    if measured.solids != split.pieces || !measured.closed { return Err(Error::Invalid("split solid compound mismatch")); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let budget = snapshot_budget(&remaining);
    let mut refusal = None;
    let piece = crate::brep::snapshot_via_with_admission(deflection, &remaining, Some(&mut |live, pending| {
        let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
            held.bytes.checked_add(live).ok_or(Error::Overflow("split piece admission"))
                .and_then(|total| admit(total, pending))
        };
        if let Err(error) = &result { refusal = Some(error.clone()); }
        result
    }), |sink, context, message, capacity| {
        // SAFETY: the compound source and callback remain live for the native call.
        unsafe { om9_occt_split_solid_piece(split.brep.source().as_ptr(), split.brep.source().len(),
            index, deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(|error| refusal.unwrap_or_else(|| Error::Io(format!("split piece: {error}"))))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let measured = crate::creation::measure_validated_source(piece.source())
        .map_err(|e| Error::Io(format!("split piece: {e}")))?;
    if measured.solids != 1 || !measured.closed || measured.volume_mm3 <= 0. { return Err(Error::Invalid("split piece is not a closed solid")); }
    let mut retained = held;
    retained.add(piece.storage_usage(&remaining)?, limits)?;
    admit(retained.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(piece)
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
    target: &dyn BrepSourceAccess,
    cutter: &dyn BrepSourceAccess,
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

fn split_live_usage(targets: &[Brep], cutters: &[Brep], vectors: &[&Vec<Brep>], limits: &Limits) -> om9_core::Result<om9_core::budget::Usage> {
    use om9_core::budget::Usage;
    let mut usage = Usage::default();
    for body in targets.iter().chain(cutters) { usage.add(body.storage_usage(limits)?, limits)?; }
    for values in vectors {
        usage.add(Usage::array::<Brep>(values.capacity())?, limits)?;
        for body in values.iter() { usage.add(body.storage_usage(limits)?, limits)?; }
    }
    Ok(usage)
}

fn split_subtract_usage(a: om9_core::budget::Usage, b: om9_core::budget::Usage) -> om9_core::Result<om9_core::budget::Usage> {
    use om9_core::{Error, budget::Usage};
    Ok(Usage {
        bytes: a.bytes.checked_sub(b.bytes).ok_or(Error::Overflow("split accounting"))?,
        objects: a.objects.checked_sub(b.objects).ok_or(Error::Overflow("split accounting"))?,
        nodes: a.nodes.checked_sub(b.nodes).ok_or(Error::Overflow("split accounting"))?,
        points: a.points.checked_sub(b.points).ok_or(Error::Overflow("split accounting"))?,
        history: a.history.checked_sub(b.history).ok_or(Error::Overflow("split accounting"))?,
        outputs: a.outputs.checked_sub(b.outputs).ok_or(Error::Overflow("split accounting"))?,
    })
}

fn split_pair_usage(a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, limits: &Limits) -> om9_core::Result<om9_core::budget::Usage> {
    let mut usage = a.storage_usage(limits)?;
    if !a.shares_validation_payload(b) { usage.add(b.storage_usage(limits)?, limits)?; }
    Ok(usage)
}

fn split_reserve_vec(values: &mut Vec<Brep>, additional: usize, live: om9_core::budget::Usage,
    limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> om9_core::Result<()> {
    use om9_core::{Error, budget::Usage};
    if values.capacity().saturating_sub(values.len()) >= additional { return Ok(()); }
    let count = values.len().checked_add(additional).ok_or(Error::Overflow("split vector"))?;
    let pending = Usage::array::<Brep>(count)?.bytes;
    let combined = live.bytes.checked_add(pending).ok_or(Error::Overflow("split vector admission"))?;
    limits.remaining_after(Usage { bytes: combined, ..Usage::default() })?;
    admit(live.bytes, pending)?;
    values.try_reserve_exact(additional).map_err(|_| Error::Budget("split vector"))?;
    Ok(())
}

/// Partition multiple materialized inputs with cancellation and incremental
/// admission. Shared bodies are conservatively counted per retained occurrence.
/// Callers must add any separately retained source leases and input-vector
/// metadata to the callback and keep the returned output reservation alive.
pub fn split_solid_sets_cancellable_admitted(
    targets: &[Brep], cutters: &[Brep], deflection_mm: f64, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>,
) -> om9_core::Result<Vec<Brep>> {
    use om9_core::Error;
    if targets.is_empty() || cutters.is_empty() { return Err(Error::Missing("split target and cutter sets")); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let deflection = checked_deflection(deflection_mm).map_err(|e| Error::Io(e.to_string()))?;
    let mut results: Vec<Brep> = Vec::new();
    let mut pending: Vec<Brep> = Vec::new();
    let mut next: Vec<Brep> = Vec::new();
    let initial = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
    admit(initial.bytes, 0)?;
    for body in targets.iter().chain(cutters) {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let body_usage = body.storage_usage(limits)?;
        let outside = split_subtract_usage(initial, body_usage)?;
        let display = body_usage.bytes.checked_sub(body.source_storage_bytes()?).ok_or(Error::Overflow("split input accounting"))?;
        let mut input_held = outside;
        input_held.add(om9_core::budget::Usage { bytes: display, ..Default::default() }, limits)?;
        let reduced = limits.remaining_after(input_held)?;
        body.validate_source_admitted(&reduced, |live, pending| {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            admit(input_held.bytes.checked_add(live).ok_or(Error::Overflow("split input admission"))?, pending)
        })?;
        let measure = crate::creation::measure_validated_source(body.source())
            .map_err(|e| Error::Io(format!("split input: {e}")))?;
        if measure.solids != 1 || !measure.closed { return Err(Error::Invalid("split needs closed single solids")); }
    }
    for target in targets {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let live = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
        split_reserve_vec(&mut pending, 1, live, limits, &mut admit)?;
        let mut after = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
        after.add(target.storage_usage(limits)?, limits)?;
        admit(after.bytes, 0)?;
        pending.push(target.clone());
        for cutter in targets.iter().chain(cutters) {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            if cutter == target { continue; }
            for candidate in &pending {
                let live = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                let outside = split_subtract_usage(live, split_pair_usage(candidate, cutter, limits)?)?;
                let reduced = limits.remaining_after(outside)?;
                let split = split_breps_cancellable_admitted(candidate, cutter, deflection, &reduced, cancel, |local, more| {
                    admit(outside.bytes.checked_add(local).ok_or(Error::Overflow("split admission"))?, more)
                })?;
                let split_usage = split.brep.storage_usage(&limits.remaining_after(live)?)?;
                let count = if split.pieces == 1 { 1 } else { split.pieces };
                if next.len().checked_add(count).ok_or(Error::Overflow("split outputs"))? as u64 > limits.max_outputs {
                    return Err(Error::Budget("split output count"));
                }
                let mut with_split = live; with_split.add(split_usage, limits)?;
                split_reserve_vec(&mut next, count, with_split, limits, &mut admit)?;
                if split.pieces == 1 {
                    let mut held = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                    held.add(split_usage, limits)?; held.add(candidate.storage_usage(limits)?, limits)?;
                    admit(held.bytes, 0)?;
                    next.push(candidate.clone());
                } else {
                    for index in 0..split.pieces {
                        let held = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                        let reduced = limits.remaining_after(held)?;
                        let piece = split_piece_cancellable_admitted(&split, index, deflection, &reduced, cancel, |local, more| {
                            admit(held.bytes.checked_add(local).ok_or(Error::Overflow("split piece admission"))?, more)
                        })?;
                        next.push(piece);
                    }
                }
                drop(split);
                let current = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                admit(current.bytes, 0)?;
            }
            std::mem::swap(&mut pending, &mut next);
            next.clear();
            let current = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
            admit(current.bytes, 0)?;
        }
        for candidate in &pending {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let mut duplicate = false;
            for existing in &results {
                let live = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                let outside = split_subtract_usage(live, split_pair_usage(candidate, existing, limits)?)?;
                let reduced = limits.remaining_after(outside)?;
                let mut compare = |left: &dyn BrepSourceAccess, right: &dyn BrepSourceAccess| boolean_breps_cancellable_admitted(
                    left, right, BooleanOp::Difference, deflection, false, &reduced, cancel, &mut |local, more| {
                        admit(outside.bytes.checked_add(local).ok_or(Error::Overflow("split duplicate admission"))?, more)
                    });
                if matches!(compare(candidate, existing)?, BooleanOutcome::Empty)
                    && matches!(compare(existing, candidate)?, BooleanOutcome::Empty) { duplicate = true; break; }
            }
            if !duplicate {
                if results.len() as u64 >= limits.max_outputs { return Err(Error::Budget("split output count")); }
                let live = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                split_reserve_vec(&mut results, 1, live, limits, &mut admit)?;
                let mut after = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
                after.add(candidate.storage_usage(limits)?, limits)?;
                admit(after.bytes, 0)?;
                results.push(candidate.clone());
            }
        }
        pending.clear();
        let current = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
        admit(current.bytes, 0)?;
    }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let final_usage = split_live_usage(targets, cutters, &[&results, &pending, &next], limits)?;
    admit(final_usage.bytes, 0)?;
    Ok(results)
}

/// Split one exact planar face with a straight trim line that crosses it.
///
/// The sum of the piece areas must equal the original face area; a trim that does not cross the
/// face is refused rather than returning the unsplit face as a "split".
pub fn split_face(
    face: &dyn BrepSourceAccess,
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

/// Remove the uniquely picked region after splitting a face by a polyline.
/// Points on a shared boundary or outside the requested tolerance are rejected.
pub fn trim_face_by_polyline(
    face: &dyn BrepSourceAccess,
    trim_line: &[Point3],
    remove_point: Point3,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    positive(tolerance_mm, "trim pick tolerance")?;
    if remove_point.iter().any(|value| !value.is_finite()) {
        return Err(EditingError::Invalid("non-finite trim pick".into()));
    }
    let split = split_face(face, trim_line, deflection_mm, limits)?;
    remove_picked_face(&split.brep, remove_point, tolerance_mm, deflection_mm, limits)?
        .ok_or_else(|| EditingError::Invalid("trim removed every face".into()))
}

/// Remove one uniquely picked face from an already partitioned open BRep.
/// Returns None when the last face was picked.
pub fn remove_picked_face(
    partition: &dyn BrepSourceAccess,
    remove_point: Point3,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Option<Brep>, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(tolerance_mm, "trim pick tolerance")?;
    if remove_point.iter().any(|value| !value.is_finite()) {
        return Err(EditingError::Invalid("non-finite trim pick".into()));
    }
    let count = checked_brep(partition, limits)?.faces;
    if count == 0 {
        return Err(EditingError::Invalid("trim partition has no faces".into()));
    }
    let mut picked = None;
    for index in 0..count {
        let piece = extract_face(partition, index, deflection_mm, limits)?;
        let distance = crate::measurement::distance(
            &om9_core::geometry::Geometry::Brep(piece.into_canonical_and_representation().0),
            om9_core::geometry::Transform::identity(),
            &om9_core::geometry::Geometry::Point(remove_point),
            om9_core::geometry::Transform::identity(),
            limits,
        )
        .map_err(|error| EditingError::Kernel(format!("trim pick distance: {error}")))?;
        if !distance.millimetres.is_finite() || distance.millimetres < 0.0 {
            return Err(EditingError::Kernel("invalid trim pick distance".into()));
        }
        if distance.millimetres <= tolerance_mm {
            if picked.replace(index).is_some() {
                return Err(EditingError::Invalid("trim pick is on multiple regions".into()));
            }
        }
    }
    let picked = picked.ok_or_else(|| EditingError::Invalid("trim pick is off the face".into()))?;
    if count == 1 { return Ok(None); }
    let index = c_int::try_from(picked).map_err(|_| EditingError::Budget("face index".into()))?;
    let budget = snapshot_budget(limits);
    let remaining = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the immutable source and callback buffers remain live until the call returns.
        unsafe { om9_occt_remove_region(partition.source().as_ptr(), partition.source().len(), index,
            deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity) }
    }).map_err(EditingError::from)?;
    if checked_brep(&remaining, limits)?.faces != count - 1 {
        return Err(EditingError::Kernel("removed region face count mismatch".into()));
    }
    Ok(Some(remaining))
}

/// Remove artificial boundaries between adjacent regions of a single original face.
/// The caller must not use this on independently authored faces whose boundaries matter.
pub fn unify_surface_regions(
    partition: &dyn BrepSourceAccess,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let before = checked_brep(partition, limits)?;
    if before.faces == 0 || before.solids != 0 {
        return Err(EditingError::Invalid("expected open surface regions".into()));
    }
    let budget = snapshot_budget(limits);
    let result = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: immutable source and callback buffers live until the synchronous call returns.
        unsafe {
            om9_occt_unify_surface_regions(partition.source().as_ptr(), partition.source().len(),
                deflection, budget.points, budget.nodes, budget.bytes, sink, context, message, capacity)
        }
    }).map_err(EditingError::from)?;
    let after = checked_brep(&result, limits)?;
    if after.faces == 0 || after.faces > before.faces || after.solids != 0
        || (after.area_mm2 - before.area_mm2).abs() > before.area_mm2.abs() * 1e-8 + 1e-9
    {
        return Err(EditingError::Kernel("unified regions changed topology or area".into()));
    }
    Ok(result)
}

/// Unify temporary region boundaries while retaining edges from the original BRep.
pub fn unify_surface_regions_preserving_edges(
    partition: &dyn BrepSourceAccess,
    original: &dyn BrepSourceAccess,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    let before = checked_brep(partition, limits)?;
    let original_measure = checked_brep(original, limits)?;
    if before.faces == 0 || before.solids != 0 || original_measure.faces == 0 {
        return Err(EditingError::Invalid("expected surface regions and original faces".into()));
    }
    let budget = snapshot_budget(limits);
    let result = snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: both sources and callback buffers live through the synchronous operation.
        unsafe {
            om9_occt_unify_surface_preserving_edges(partition.source().as_ptr(), partition.source().len(),
                original.source().as_ptr(), original.source().len(), deflection,
                budget.points, budget.nodes, budget.bytes, sink, context, message, capacity)
        }
    }).map_err(EditingError::from)?;
    let after = checked_brep(&result, limits)?;
    if after.faces == 0 || after.faces > before.faces || after.solids != 0
        || (after.area_mm2 - before.area_mm2).abs() > before.area_mm2.abs() * 1e-8 + 1e-9
    {
        return Err(EditingError::Kernel("unified regions changed topology or area".into()));
    }
    Ok(result)
}

/* ------------------------- cap ------------------------- */

/// Cap supported planar boundary loops, retaining any non-planar openings.
pub fn cap_planar(
    open: &dyn BrepSourceAccess,
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
    solid: &dyn BrepSourceAccess,
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
    let table = edge_table(edges, solid.topology().edges.len())?;
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
    solid: &dyn BrepSourceAccess,
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
    solid: &dyn BrepSourceAccess,
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
        if index >= solid.topology().edges.len() || indices.contains(&native) {
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
pub fn straight_edge_endpoints(shape: &dyn BrepSourceAccess, index: usize, limits: &Limits) -> Result<(Point3, Point3), EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.topology().edges.len() { return Err(EditingError::Invalid("edge index out of range".into())); }
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
pub fn verified_edge_endpoints(shape: &dyn BrepSourceAccess, segment: [Point3; 2], display_deflection: f64, segment_tolerance: f64,
    limits: &Limits) -> Result<Option<(Point3, Point3)>, EditingError> {
    shape.validate_source(limits).map_err(|error| EditingError::Invalid(error.to_string()))?;
    if !display_deflection.is_finite() || display_deflection <= 0.0 || !segment_tolerance.is_finite() || segment_tolerance <= 0.0 || segment.iter().flatten().any(|v| !v.is_finite()) {
        return Err(EditingError::Invalid("fillet display segment".into()));
    }
    let mut points = [0.; 6];
    let mut message = [0 as c_char; 512];
    let status = unsafe { om9_occt_verified_edge_endpoints(shape.source().as_ptr(), shape.source().len(), segment.as_ptr().cast(),
        display_deflection + segment_tolerance, segment_tolerance,
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX), points.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status == 3 { return Ok(None); }
    if status != 0 { return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned())); }
    if !points.iter().all(|v| v.is_finite()) { return Err(EditingError::Kernel("invalid edge endpoints".into())); }
    Ok(Some(([points[0], points[1], points[2]], [points[3], points[4], points[5]])))
}

/// Retrieve exact endpoints in the orientation used by `fillet_with_end_radii`.
pub fn oriented_edge_endpoints(
    shape: &dyn BrepSourceAccess,
    index: usize,
    limits: &Limits,
) -> Result<(Point3, Point3), EditingError> {
    oriented_edge_endpoints_or_closed(shape, index, limits)?
        .ok_or_else(|| EditingError::Invalid("edge has a single closed endpoint".into()))
}

/// Return oriented endpoints, or None only for an edge with one shared topological vertex.
pub fn oriented_edge_endpoints_or_closed(
    shape: &dyn BrepSourceAccess,
    index: usize,
    limits: &Limits,
) -> Result<Option<(Point3, Point3)>, EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.topology().edges.len() {
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
pub fn circular_edge(shape: &dyn BrepSourceAccess, index: usize, limits: &Limits) -> Result<om9_core::geometry::Curve, EditingError> {
    checked_brep(shape, limits)?;
    if index >= shape.topology().edges.len() { return Err(EditingError::Invalid("edge index out of range".into())); }
    let index = i32::try_from(index).map_err(|_| EditingError::Budget("edge index".into()))?;
    let mut values = [0.0; 13];
    let mut message = [0 as c_char; 512];
    // SAFETY: all fixed buffers and BRep bytes stay alive during the synchronous native query.
    let status = unsafe { om9_occt_circular_edge(shape.source().as_ptr(), shape.source().len(), index,
        values.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 {
        return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
    }
    circle_from_native(&values, limits)
}

fn circle_from_native(values: &[f64], limits: &Limits) -> Result<om9_core::geometry::Curve, EditingError> {
    if values.len() != 13 || !values.iter().all(|v| v.is_finite()) || values[9] <= 0.0 || values[10] >= values[11] || values[12] != 0.0 && values[12] != 1.0 {
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
pub fn fillet_project_edge(shape: &dyn BrepSourceAccess, edge: usize, point: Point3, limits: &Limits) -> Result<(f64, Point3), EditingError> {
    fillet_project_edge_inner(shape, edge, point, None, limits)
}

pub fn fillet_project_verified_edge(shape: &dyn BrepSourceAccess, edge: usize, point: Point3, segment: [Point3; 2],
    display_deflection: f64, tolerance: f64, limits: &Limits) -> Result<(f64, Point3), EditingError> {
    validate_fillet_segments(&[segment], tolerance)?;
    checked_deflection(display_deflection)?;
    fillet_project_edge_inner(shape, edge, point, Some((segment, display_deflection, tolerance)), limits)
}

fn fillet_project_edge_inner(shape: &dyn BrepSourceAccess, edge: usize, point: Point3, segment: Option<([Point3; 2], f64, f64)>, limits: &Limits) -> Result<(f64, Point3), EditingError> {
    checked_brep(shape, limits)?;
    if edge >= shape.topology().edges.len() || !point.iter().all(|x| x.is_finite()) {
        return Err(EditingError::Invalid("fillet edge projection input".into()));
    }
    let edge = i32::try_from(edge).map_err(|_| EditingError::Budget("fillet edge index".into()))?;
    let mut output = [0.0; 4];
    let mut message = [0 as c_char; 512];
    // SAFETY: the source, point and fixed output buffers remain alive for this synchronous call.
    let status = unsafe { om9_occt_fillet_project_edge(shape.source().as_ptr(), shape.source().len(), edge,
        point.as_ptr(), segment.as_ref().map_or(std::ptr::null(), |(s, _, _)| s.as_ptr().cast()),
        segment.as_ref().map_or(0.0, |(_, d, t)| d + t),
        segment.as_ref().map_or(0.0, |(_, _, t)| *t), usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        output.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
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
pub fn fillet_shared_endpoint(shape: &dyn BrepSourceAccess, anchor: usize, side: usize, candidate: usize, limits: &Limits) -> Result<u8, EditingError> {
    Ok(fillet_shared_endpoints(shape, anchor, side, &[candidate], limits)?[0])
}

/// Classify a batch against one contour, reading and validating the source only once.
pub fn fillet_shared_endpoints(shape: &dyn BrepSourceAccess, anchor: usize, side: usize, candidates: &[usize], limits: &Limits) -> Result<Vec<u8>, EditingError> {
    fillet_shared_endpoints_inner(shape, anchor, side, candidates, None, limits)
}

pub fn fillet_shared_verified_endpoints(shape: &dyn BrepSourceAccess, anchor: usize, side: usize, candidates: &[usize],
    segments: &[[Point3; 2]], anchor_slot: usize, display_deflection: f64, tolerance: f64, limits: &Limits) -> Result<Vec<u8>, EditingError> {
    if segments.len() != candidates.len() || anchor_slot >= segments.len() { return Err(EditingError::Invalid("fillet segment count".into())); }
    validate_fillet_segments(segments, tolerance)?;
    checked_deflection(display_deflection)?;
    fillet_shared_endpoints_inner(shape, anchor, side, candidates, Some((segments, anchor_slot, display_deflection, tolerance)), limits)
}

fn validate_fillet_segments(segments: &[[Point3; 2]], tolerance: f64) -> Result<(), EditingError> {
    if !tolerance.is_finite() || tolerance <= 0.0 || segments.iter().flatten().flatten().any(|v| !v.is_finite()) {
        return Err(EditingError::Invalid("invalid fillet display segment".into()));
    }
    Ok(())
}

fn fillet_shared_endpoints_inner(shape: &dyn BrepSourceAccess, anchor: usize, side: usize, candidates: &[usize],
    segments: Option<(&[[Point3; 2]], usize, f64, f64)>, limits: &Limits) -> Result<Vec<u8>, EditingError> {
    checked_brep(shape, limits)?;
    if anchor >= shape.topology().edges.len() || candidates.is_empty() || candidates.iter().any(|&i| i >= shape.topology().edges.len()) || side > 1 {
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
        side as c_int, native.as_ptr(), native.len(), segments.map_or(std::ptr::null(), |(s, _, _, _)| s.as_ptr().cast()),
        segments.map_or(0, |(_, slot, _, _)| slot), segments.map_or(0.0, |(_, _, d, t)| d + t),
        segments.map_or(0.0, |(_, _, _, t)| t), usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
        masks.as_mut_ptr(), message.as_mut_ptr(), message.len() as c_int) };
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
    shape: &dyn BrepSourceAccess,
    edges: &[(usize, f64, f64)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    fillet_with_radius_controls(shape, edges, &[], deflection_mm, limits)
}

/// Radius controls use an edge index, oriented arc-length fraction in (0,1), and radius.
/// The contour must be open and fully selected; OCCT interpolates the supplied law.
pub fn fillet_with_radius_controls(
    shape: &dyn BrepSourceAccess,
    edges: &[(usize, f64, f64)],
    controls: &[(usize, f64, f64)],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    fillet_with_radius_controls_inner(shape, edges, controls, None, deflection_mm, limits)
}

/// Resolve each displayed segment against the same native shape used by the fillet.
pub fn fillet_with_verified_segments(
    shape: &dyn BrepSourceAccess, edges: &[(usize, f64, f64)], controls: &[(usize, f64, f64)],
    segments: &[[Point3; 2]], display_deflection: f64, segment_tolerance: f64, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    if segments.len() != edges.len() || !segment_tolerance.is_finite() || segment_tolerance <= 0.0
        || segments.iter().flatten().flatten().any(|v| !v.is_finite()) {
        return Err(EditingError::Invalid("invalid fillet display segments".into()));
    }
    checked_deflection(display_deflection)?;
    fillet_with_radius_controls_inner(shape, edges, controls, Some((segments, display_deflection, segment_tolerance)), deflection_mm, limits)
}

fn fillet_with_radius_controls_inner(
    shape: &dyn BrepSourceAccess, edges: &[(usize, f64, f64)], controls: &[(usize, f64, f64)],
    segments: Option<(&[[Point3; 2]], f64, f64)>, deflection_mm: f64, limits: &Limits,
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
        if index >= shape.topology().edges.len() || indices.contains(&native) {
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
                segments.map_or(std::ptr::null(), |(segments, _, _)| segments.as_ptr().cast::<f64>()),
                segments.map_or(0.0, |(_, display_deflection, tolerance)| display_deflection + tolerance),
                segments.map_or(0.0, |(_, _, tolerance)| tolerance),
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
    solid: &dyn BrepSourceAccess,
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
    if edge_index >= brep.display().edges.len() {
        return Err(EditingError::Invalid(format!(
            "edge index {edge_index} is outside the {} published edges",
            brep.display().edges.len()
        )));
    }
    let mut first: Option<Point3> = None;
    let mut last: Option<Point3> = None;
    for [a, b, edge] in &brep.display().segments {
        if *edge as usize != edge_index {
            continue;
        }
        if first.is_none() {
            first = Some(brep.display().points[*a as usize]);
        }
        last = Some(brep.display().points[*b as usize]);
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
    shape: &dyn BrepSourceAccess,
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
    shape: &dyn BrepSourceAccess,
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
    brep: &dyn BrepSourceAccess,
    face_index: usize,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    extract_face_inner(brep, face_index, None, deflection_mm, limits, None)
}

pub fn extract_verified_face(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    extract_verified_face_with_witness_deflection(brep, face_index, triangle, vertex_tolerance, deflection_mm, deflection_mm, limits)
}

pub fn extract_verified_face_with_witness_deflection(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    witness_deflection_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], vertex_tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection_mm)?;
    extract_face_inner(brep, face_index, Some((triangle, vertex_tolerance, witness_deflection)), deflection_mm, limits, None)
}

/// Admit Rust output and snapshot scratch; OCCT internal allocations are not covered.
pub fn extract_verified_face_with_witness_deflection_admitted(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    witness_deflection_mm: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], vertex_tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection_mm)?;
    extract_face_inner(brep, face_index, Some((triangle, vertex_tolerance, witness_deflection)), deflection_mm, limits, Some(&mut admit))
}

fn validate_face_triangles(triangles: &[[Point3; 3]], tolerance: f64) -> Result<(), EditingError> {
    if !tolerance.is_finite() || tolerance <= 0.0 || triangles.iter().flatten().flatten().any(|v| !v.is_finite()) {
        return Err(EditingError::Invalid("invalid face display triangle".into()));
    }
    Ok(())
}

/// Bind all triangles labelled as one display face to the same source face.
pub fn verify_face_display(brep: &dyn BrepSourceAccess, display: &om9_core::brep::BrepDisplay, face: usize,
    witness: [Point3; 3], tolerance: f64, limits: &Limits) -> Result<(), EditingError> {
    brep.validate_source(limits).map_err(|error| EditingError::Invalid(error.to_string()))?;
    verify_face_display_inner(brep.source(), display, face, witness, tolerance, limits, None)
        .map_err(|error| match error { FaceCheckError::Canceled => EditingError::Kernel("face verification canceled".into()),
            FaceCheckError::Editing(other) => other })
}

pub fn verify_face_display_cancellable(brep: &dyn BrepSourceAccess, display: &om9_core::brep::BrepDisplay, face: usize,
    witness: [Point3; 3], tolerance: f64, limits: &Limits, cancel: &om9_core::transaction::Cancellation) -> om9_core::Result<()> {
    if cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
    brep.validate_source(limits).map_err(|error| om9_core::Error::Io(error.to_string()))?;
    verify_face_display_inner(brep.source(), display, face, witness, tolerance, limits, Some(cancel))
        .map_err(|error| match error { FaceCheckError::Canceled => om9_core::Error::Canceled,
            FaceCheckError::Editing(other) => om9_core::Error::Io(other.to_string()) })
}

/// Verify against a separately tracked canonical source without capturing its
/// document or display. Admission includes source-validation scratch.
pub fn verify_captured_face_display_cancellable(source: &om9_core::transaction::CapturedBrepSource,
    display: &om9_core::brep::BrepDisplay, face: usize, witness: [Point3; 3], tolerance: f64,
    limits: &Limits, cancel: &om9_core::transaction::Cancellation,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> om9_core::Result<()> {
    if cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
    let canonical = source.validate_canonical_admitted(limits, &mut admit)?;
    match verify_face_display_inner(source.source(), display, face, witness, tolerance, limits, Some(cancel)) {
        Ok(()) => Ok(()),
        Err(FaceCheckError::Canceled) => Err(om9_core::Error::Canceled),
        Err(FaceCheckError::Editing(EditingError::Kernel(message)))
            if message.contains("source face display: triangle crosses trim boundary") => {
                verify_captured_face_exact_remesh(source, display, face, witness, tolerance, limits, canonical, cancel, &mut admit)
            }
        Err(FaceCheckError::Editing(other)) => Err(om9_core::Error::Io(other.to_string())),
    }
}

/// A fresh source mesh is an alternative binding witness only when it matches
/// the entire displayed face mesh exactly; it does not make display facets
/// exact source geometry. Callers must still refine the actual picked point.
fn verify_captured_face_exact_remesh(source: &om9_core::transaction::CapturedBrepSource,
    display: &om9_core::brep::BrepDisplay, face: usize, witness: [Point3; 3], tolerance: f64,
    limits: &Limits, canonical: om9_core::budget::Usage, cancel: &om9_core::transaction::Cancellation,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> om9_core::Result<()> {
    use om9_core::Error;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if display.source_sha256 != source.source_fingerprint() || &*display.topology != source.topology()
        || face >= source.topology().faces.len() {
        return Err(Error::Stale("face remesh source binding"));
    }
    let remaining = limits.remaining_after(canonical)?;
    let mut admission_error = None;
    let remesh = crate::brep::capture_source_topology(source.source(), display.deflection_mm, 0.3, &remaining,
        |live, pending| {
            let result = if cancel.is_canceled() { Err(Error::Canceled) } else {
                canonical.bytes.checked_add(live).ok_or(Error::Overflow("face remesh admission"))
                    .and_then(|held| admit(held, pending))
            };
            if let Err(error) = &result { admission_error = Some(error.clone()); }
            result
        });
    if let Some(error) = admission_error { return Err(error); }
    let remesh = remesh.map_err(|error| Error::Io(format!("face source remesh: {error}")))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    let fresh = remesh.brep().display();
    let topology = source.topology();
    if fresh.topology.faces.len() != topology.faces.len() || fresh.topology.edges.len() != topology.edges.len()
        || fresh.topology.vertex_ids.len() != topology.vertex_ids.len() || fresh.topology.face_edges != topology.face_edges
        || fresh.points.len() != display.points.len() || fresh.triangles.len() != display.triangles.len() {
        return Err(Error::Stale("face remesh topology or display"));
    }
    for (old, new) in display.points.chunks(4096).zip(fresh.points.chunks(4096)) {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if old != new { return Err(Error::Stale("face remesh vertices")); }
    }
    let mut found = false;
    for (old, new) in display.triangles.chunks(4096).zip(fresh.triangles.chunks(4096)) {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        if old != new { return Err(Error::Stale("face remesh triangles")); }
        for triangle in old.iter().filter(|triangle| triangle[3] as usize == face) {
            let Some(a) = display.points.get(triangle[0] as usize) else { return Err(Error::Stale("face remesh vertex")) };
            let Some(b) = display.points.get(triangle[1] as usize) else { return Err(Error::Stale("face remesh vertex")) };
            let Some(c) = display.points.get(triangle[2] as usize) else { return Err(Error::Stale("face remesh vertex")) };
            if [*a, *b, *c] == witness { found = true; }
        }
    }
    if !found { return Err(Error::Stale("face remesh witness")); }
    drop(remesh);
    admit(canonical.bytes, 0)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    verify_face_triangles_source(source.source(), &[witness], tolerance, display.deflection_mm, limits)
        .map_err(|error| Error::Io(format!("face remesh witness: {error}")))?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok(())
}

enum FaceCheckError { Canceled, Editing(EditingError) }

fn verify_face_display_inner(source: &str, display: &om9_core::brep::BrepDisplay, face: usize,
    witness: [Point3; 3], tolerance: f64, limits: &Limits, cancel: Option<&om9_core::transaction::Cancellation>) -> Result<(), FaceCheckError> {
    if cancel.is_some_and(|token| token.is_canceled()) { return Err(FaceCheckError::Canceled); }
    validate_face_triangles(&[witness], tolerance).map_err(FaceCheckError::Editing)?;
    let deflection = checked_deflection(display.deflection_mm).map_err(FaceCheckError::Editing)?;
    let face = u32::try_from(face).map_err(|_| FaceCheckError::Editing(EditingError::Invalid("display face index".into())))?;
    let mut message = [0 as c_char; 512];
    unsafe extern "C" fn is_canceled(context: *mut c_void) -> c_int {
        // SAFETY: caller supplies a live Cancellation for this synchronous call.
        let token = unsafe { &*(context.cast::<om9_core::transaction::Cancellation>()) };
        c_int::from(token.is_canceled())
    }
    // SAFETY: all slices and the output buffer live for this synchronous call;
    // Point3 and [u32; 4] are contiguous fixed-size arrays.
    let status = unsafe { om9_occt_verify_face_display(source.as_ptr(), source.len(),
        witness.as_ptr().cast(), deflection, tolerance, display.points.as_ptr().cast(), display.points.len(),
        display.triangles.as_ptr().cast(), display.triangles.len(), face,
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX), cancel.map(|_| is_canceled as unsafe extern "C" fn(*mut c_void) -> c_int),
        cancel.map_or(std::ptr::null_mut(), |token| token as *const _ as *mut c_void), message.as_mut_ptr(), message.len() as c_int) };
    if status == 3 { return Err(FaceCheckError::Canceled); }
    if status != 0 { return Err(FaceCheckError::Editing(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()))); }
    Ok(())
}

/// Reject ambiguous or duplicate face witnesses before an atomic multi-face edit.
pub fn verify_source_face_triangles(brep: &Brep, triangles: &[[Point3; 3]], tolerance: f64,
    limits: &Limits) -> Result<(), EditingError> {
    verify_source_face_triangles_with_deflection(brep, triangles, tolerance, brep.display().deflection_mm, limits)
}
pub fn verify_source_face_triangles_with_deflection(brep: &dyn BrepSourceAccess, triangles: &[[Point3; 3]], tolerance: f64,
    witness_deflection: f64, limits: &Limits) -> Result<(), EditingError> {
    validate_face_triangles(triangles, tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection)?;
    brep.validate_source(limits).map_err(|error| EditingError::Invalid(error.to_string()))?;
    verify_face_triangles_source(brep.source(), triangles, tolerance, witness_deflection, limits)
}

fn captured_source_measure(source: &om9_core::transaction::CapturedBrepSource, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Measures, EditingError> {
    source.validate_canonical_admitted(limits, admit).map_err(|error| match error {
        om9_core::Error::Budget(_) => EditingError::Budget(error.to_string()),
        _ => EditingError::Invalid(error.to_string()),
    })?;
    crate::creation::measure_validated_source(source.source())
}

pub fn captured_source_face_count_admitted(source: &om9_core::transaction::CapturedBrepSource, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<usize, EditingError> {
    Ok(captured_source_measure(source, limits, &mut admit)?.faces)
}

pub fn verify_captured_face_triangles_admitted(source: &om9_core::transaction::CapturedBrepSource,
    triangles: &[[Point3; 3]], tolerance: f64, witness_deflection: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<(), EditingError> {
    validate_face_triangles(triangles, tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection)?;
    source.validate_canonical_admitted(limits, &mut admit).map_err(|error| match error {
        om9_core::Error::Budget(_) => EditingError::Budget(error.to_string()),
        _ => EditingError::Invalid(error.to_string()),
    })?;
    verify_face_triangles_source(source.source(), triangles, tolerance, witness_deflection, limits)
}

fn verify_face_triangles_source(source: &str, triangles: &[[Point3; 3]], tolerance: f64,
    witness_deflection: f64, limits: &Limits) -> Result<(), EditingError> {
    if triangles.is_empty() || triangles.len() as u64 > limits.max_outputs { return Err(EditingError::Budget("face witness count".into())); }
    let mut message = [0 as c_char; 512];
    let status = unsafe { om9_occt_verify_face_triangles(source.as_ptr(), source.len(), triangles.as_ptr().cast(),
        triangles.len(), witness_deflection + tolerance, tolerance,
        usize::try_from(limits.max_nodes).unwrap_or(usize::MAX), usize::try_from(limits.max_bytes).unwrap_or(usize::MAX),
        message.as_mut_ptr(), message.len() as c_int) };
    if status != 0 { return Err(EditingError::Kernel(unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned())); }
    Ok(())
}

fn extract_face_inner(brep: &dyn BrepSourceAccess, face_index: usize, triangle: Option<([Point3; 3], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    extract_face_source(brep.source(), before, face_index, triangle, deflection_mm, limits, admission)
}

pub fn extract_captured_face_admitted(source: &om9_core::transaction::CapturedBrepSource, face_index: usize,
    triangle: [Point3; 3], tolerance: f64, witness_deflection: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection)?;
    checked_deflection(deflection_mm)?;
    let before = captured_source_measure(source, limits, &mut admit)?;
    extract_face_source(source.source(), before, face_index, Some((triangle, tolerance, witness_deflection)), deflection_mm, limits, Some(&mut admit))
}

fn extract_face_source(source: &str, before: Measures, face_index: usize, triangle: Option<([Point3; 3], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    if face_index >= before.faces {
        return Err(EditingError::Invalid(format!(
            "face index {face_index} is outside the {} faces",
            before.faces
        )));
    }
    let native_index =
        i32::try_from(face_index).map_err(|_| EditingError::Budget("face index".into()))?;
    let budget = snapshot_budget(limits);
    let face = snapshot_via_with_admission(deflection, limits, admission, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_extract_face(
                source.as_ptr(),
                source.len(),
                native_index,
                triangle.as_ref().map_or(std::ptr::null(), |(t, _, _)| t.as_ptr().cast()),
                triangle.as_ref().map_or(0.0, |(_, t, _)| *t),
                triangle.as_ref().map_or(0.0, |(_, _, d)| *d),
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
    brep: &dyn BrepSourceAccess,
    face_index: usize,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    remove_face_inner(brep, face_index, None, deflection_mm, limits, None)
}

pub fn remove_verified_face(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    remove_verified_face_with_deflection(brep, face_index, triangle, vertex_tolerance, deflection_mm, deflection_mm, limits)
}
pub fn remove_verified_face_with_deflection(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    witness_deflection: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], vertex_tolerance)?;
    remove_face_inner(brep, face_index, Some((triangle, vertex_tolerance, checked_deflection(witness_deflection)?)), deflection_mm, limits, None)
}

/// Admit Rust output and snapshot scratch; OCCT internal allocations are not covered.
pub fn remove_verified_face_with_deflection_admitted(brep: &dyn BrepSourceAccess, face_index: usize, triangle: [Point3; 3], vertex_tolerance: f64,
    witness_deflection: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], vertex_tolerance)?;
    remove_face_inner(brep, face_index, Some((triangle, vertex_tolerance, checked_deflection(witness_deflection)?)), deflection_mm, limits, Some(&mut admit))
}

fn remove_face_inner(brep: &dyn BrepSourceAccess, face_index: usize, triangle: Option<([Point3; 3], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    remove_face_source(brep.source(), before, face_index, triangle, deflection_mm, limits, admission)
}

pub fn remove_captured_face_admitted(source: &om9_core::transaction::CapturedBrepSource, face_index: usize,
    triangle: [Point3; 3], tolerance: f64, witness_deflection: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    validate_face_triangles(&[triangle], tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection)?;
    checked_deflection(deflection_mm)?;
    let before = captured_source_measure(source, limits, &mut admit)?;
    remove_face_source(source.source(), before, face_index, Some((triangle, tolerance, witness_deflection)), deflection_mm, limits, Some(&mut admit))
}

fn remove_face_source(source: &str, before: Measures, face_index: usize, triangle: Option<([Point3; 3], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    if face_index >= before.faces {
        return Err(EditingError::Invalid(format!(
            "face index {face_index} is outside the {} faces",
            before.faces
        )));
    }
    let native_index =
        i32::try_from(face_index).map_err(|_| EditingError::Budget("face index".into()))?;
    let budget = snapshot_budget(limits);
    let shell = snapshot_via_with_admission(deflection, limits, admission, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_remove_face(
                source.as_ptr(),
                source.len(),
                native_index,
                triangle.as_ref().map_or(std::ptr::null(), |(t, _, _)| t.as_ptr().cast()),
                triangle.as_ref().map_or(0.0, |(_, t, _)| *t),
                triangle.as_ref().map_or(0.0, |(_, _, d)| *d),
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
    brep: &dyn BrepSourceAccess,
    face_indices: &[usize],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    remove_faces_inner(brep, face_indices, None, deflection_mm, limits, None)
}

pub fn remove_verified_faces(brep: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    remove_verified_faces_with_deflection(brep, face_indices, triangles, vertex_tolerance, deflection_mm, deflection_mm, limits)
}
pub fn remove_verified_faces_with_deflection(brep: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    witness_deflection: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("face triangle count".into())); }
    validate_face_triangles(triangles, vertex_tolerance)?;
    remove_faces_inner(brep, face_indices, Some((triangles, vertex_tolerance, checked_deflection(witness_deflection)?)), deflection_mm, limits, None)
}

/// Admit Rust output and snapshot scratch; OCCT internal allocations are not covered.
pub fn remove_verified_faces_with_deflection_admitted(brep: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    witness_deflection: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("face triangle count".into())); }
    validate_face_triangles(triangles, vertex_tolerance)?;
    let scratch = face_indices.len().checked_mul(std::mem::size_of::<i32>())
        .and_then(|bytes| u64::try_from(bytes).ok()).ok_or_else(|| EditingError::Budget("face indices".into()))?;
    admit(0, scratch).map_err(|error| EditingError::Budget(error.to_string()))?;
    let mut account = |live: u64, pending: u64| {
        admit(live.checked_add(scratch).ok_or(om9_core::Error::Overflow("face indices"))?, pending)
    };
    remove_faces_inner(brep, face_indices, Some((triangles, vertex_tolerance, checked_deflection(witness_deflection)?)), deflection_mm, limits, Some(&mut account))
}

fn remove_faces_inner(brep: &dyn BrepSourceAccess, face_indices: &[usize], triangles: Option<(&[[Point3; 3]], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    checked_deflection(deflection_mm)?;
    let before = checked_brep(brep, limits)?;
    remove_faces_source(brep.source(), before, face_indices, triangles, deflection_mm, limits, admission)
}

pub fn remove_captured_faces_admitted(source: &om9_core::transaction::CapturedBrepSource, face_indices: &[usize],
    triangles: &[[Point3; 3]], tolerance: f64, witness_deflection: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("face triangle count".into())); }
    validate_face_triangles(triangles, tolerance)?;
    let witness_deflection = checked_deflection(witness_deflection)?;
    checked_deflection(deflection_mm)?;
    let scratch = face_indices.len().checked_mul(std::mem::size_of::<i32>())
        .and_then(|bytes| u64::try_from(bytes).ok()).ok_or_else(|| EditingError::Budget("face indices".into()))?;
    admit(0, scratch).map_err(|error| EditingError::Budget(error.to_string()))?;
    let mut account = |live: u64, pending: u64| {
        admit(live.checked_add(scratch).ok_or(om9_core::Error::Overflow("face indices"))?, pending)
    };
    let before = captured_source_measure(source, limits, &mut account)?;
    remove_faces_source(source.source(), before, face_indices, Some((triangles, tolerance, witness_deflection)), deflection_mm, limits, Some(&mut account))
}

fn remove_faces_source(source: &str, before: Measures, face_indices: &[usize], triangles: Option<(&[[Point3; 3]], f64, f64)>,
    deflection_mm: f64, limits: &Limits, admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
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
    let shell = snapshot_via_with_admission(
        deflection,
        limits,
        admission,
        |sink, context, message, capacity| unsafe {
            // SAFETY: source, index slice, and callback context live through the call.
            om9_occt_remove_faces(
                source.as_ptr(),
                source.len(),
                indices.as_ptr(),
                indices.len(),
                triangles.map_or(std::ptr::null(), |(t, _, _)| t.as_ptr().cast()),
                triangles.map_or(0.0, |(_, tolerance, _)| tolerance),
                triangles.map_or(0.0, |(_, _, d)| d),
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
    a: &dyn BrepSourceAccess,
    b: &dyn BrepSourceAccess,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    sew_with_orientation(a, b, false, false, tolerance_mm, deflection_mm, limits)
}

/// Sew only when every input face keeps its exact native supporting surface.
pub fn sew_preserving_surfaces(
    a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, tolerance_mm: f64, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    sew_with_orientation(a,b,false,true,tolerance_mm,deflection_mm,limits)
}

/// Sew shapes while reversing the second shape's topology before sewing.
pub fn sew_reversing_second(
    a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, tolerance_mm: f64, deflection_mm: f64, limits: &Limits,
) -> Result<Brep, EditingError> {
    sew_with_orientation(a, b, true, false, tolerance_mm, deflection_mm, limits)
}

fn sew_with_orientation(
    a: &dyn BrepSourceAccess, b: &dyn BrepSourceAccess, reverse_b: bool, preserve_surfaces: bool, tolerance_mm: f64,
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
                i32::from(preserve_surfaces),
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

/// Faces made by add-ons — each a NURBS surface with its trim loops, as for
/// [`crate::creation::trimmed_face`] — sewn in one pass at `tolerance_mm` into one body: a closed shell
/// becomes a solid. With `closed`, anything but exactly one closed solid of positive volume is
/// refused (a Solid=Yes result never arrives half open).
pub fn sew_faces(
    faces: &[(om9_core::geometry::NurbsSurface, Vec<Vec<om9_core::geometry::NurbsCurve>>)],
    closed: bool,
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    positive(tolerance_mm, "sew tolerance")?;
    if faces.is_empty() {
        return Err(EditingError::Invalid("a body needs at least one face".into()));
    }
    let max = usize::try_from(limits.max_points).unwrap_or(usize::MAX);
    if faces.len() > max {
        return Err(EditingError::Budget("face count".into()));
    }
    let parts = faces
        .iter()
        .map(|(surface, loops)| crate::creation::trimmed_face(surface, loops, deflection_mm, limits))
        .collect::<Result<Vec<_>, _>>()?;
    let body = if parts.len() == 1 {
        parts.into_iter().next().expect("one part")
    } else {
        sew_many(&parts, tolerance_mm, deflection_mm, limits)?
    };
    if closed {
        let m = measure(&body, limits)?;
        if m.solids != 1 || !m.closed || m.naked_edges != 0 || m.volume_mm3 <= 0.0 {
            return Err(EditingError::Invalid(format!(
                "the faces do not close into one solid ({} naked edges)",
                m.naked_edges
            )));
        }
    }
    Ok(body)
}

/// Sew many parts in one pass: one native sewing over all of them and one snapshot (a closed
/// shell becomes a solid). Unlike [`sew_all`], the time stays near linear in the part count.
pub fn sew_many(
    parts: &[Brep],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    if parts.len() < 2 {
        return Err(EditingError::Invalid("join needs at least two parts".into()));
    }
    let deflection = checked_deflection(deflection_mm)?;
    positive(tolerance_mm, "sew tolerance")?;
    for part in parts {
        checked_brep(part, limits)?;
    }
    let sources: Vec<*const u8> = parts.iter().map(|p| p.source().as_ptr()).collect();
    let lengths: Vec<usize> = parts.iter().map(|p| p.source().len()).collect();
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: the parts (and so their sources), both arrays and the callback context live
        // through this call; the arrays hold `parts.len()` entries.
        unsafe {
            om9_occt_sew_many(
                sources.as_ptr(),
                lengths.as_ptr(),
                parts.len(),
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
pub fn geometry_health(brep: &dyn BrepSourceAccess, limits: &Limits) -> Result<GeometryHealth, EditingError> {
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
        source_sha256: brep.canonical_fingerprints().0,
    })
}

unsafe extern "C" {
    fn om9_occt_shell(
        source: *const u8,
        length: usize,
        face_indices: *const usize,
        face_count: usize,
        triangles: *const f64,
        vertex_tolerance: f64,
        witness_deflection: f64,
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
    solid: &dyn BrepSourceAccess,
    face_index: usize,
    thickness_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    shell_faces(solid, &[face_index], thickness_mm, deflection_mm, limits)
}

/// Hollow a closed solid, removing the specified distinct faces in one operation.
pub fn shell_faces(
    solid: &dyn BrepSourceAccess,
    face_indices: &[usize],
    thickness_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, EditingError> {
    shell_faces_inner(solid, face_indices, None, thickness_mm, deflection_mm, limits, None)
}

pub fn shell_verified_faces(solid: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    thickness_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    shell_verified_faces_with_deflection(solid, face_indices, triangles, vertex_tolerance, deflection_mm, thickness_mm, deflection_mm, limits)
}
pub fn shell_verified_faces_with_deflection(solid: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    witness_deflection: f64, thickness_mm: f64, deflection_mm: f64, limits: &Limits) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("shell face triangle count".into())); }
    validate_face_triangles(triangles, vertex_tolerance)?;
    shell_faces_inner(solid, face_indices, Some((triangles, vertex_tolerance, checked_deflection(witness_deflection)?)), thickness_mm, deflection_mm, limits, None)
}

/// Admit Rust snapshot output and scratch before allocation. OCCT's internal
/// construction and snapshot allocations are outside this callback.
pub fn shell_verified_faces_with_deflection_admitted(solid: &dyn BrepSourceAccess, face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64,
    witness_deflection: f64, thickness_mm: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("shell face triangle count".into())); }
    validate_face_triangles(triangles, vertex_tolerance)?;
    shell_faces_inner(solid, face_indices, Some((triangles, vertex_tolerance, checked_deflection(witness_deflection)?)),
        thickness_mm, deflection_mm, limits, Some(&mut admit))
}

/// Source-only counterpart; the caller retains the tracked source lease. Rust
/// validation and snapshot scratch use `admit`; native OCCT allocations do not.
pub fn shell_captured_faces_with_deflection_admitted(source: &om9_core::transaction::CapturedBrepSource,
    face_indices: &[usize], triangles: &[[Point3; 3]], vertex_tolerance: f64, witness_deflection: f64,
    thickness_mm: f64, deflection_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<Brep, EditingError> {
    if triangles.len() != face_indices.len() { return Err(EditingError::Invalid("shell face triangle count".into())); }
    validate_face_triangles(triangles, vertex_tolerance)?;
    checked_deflection(deflection_mm)?;
    positive(thickness_mm, "shell thickness")?;
    source.validate_canonical_admitted(limits, &mut admit).map_err(|error| match error {
        om9_core::Error::Budget(_) => EditingError::Budget(error.to_string()),
        _ => EditingError::Invalid(error.to_string()),
    })?;
    let before = crate::creation::measure_validated_source(source.source())?;
    shell_faces_source_inner(source.source(), before, face_indices,
        Some((triangles, vertex_tolerance, checked_deflection(witness_deflection)?)),
        thickness_mm, deflection_mm, limits, Some(&mut admit))
}

fn shell_faces_inner(solid: &dyn BrepSourceAccess, face_indices: &[usize], triangles: Option<(&[[Point3; 3]], f64, f64)>,
    thickness_mm: f64, deflection_mm: f64, limits: &Limits,
    admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    checked_deflection(deflection_mm)?;
    positive(thickness_mm, "shell thickness")?;
    let before = checked_brep(solid, limits)?;
    shell_faces_source_inner(solid.source(), before, face_indices, triangles, thickness_mm, deflection_mm, limits, admission)
}

fn shell_faces_source_inner(source: &str, before: Measures, face_indices: &[usize], triangles: Option<(&[[Point3; 3]], f64, f64)>,
    thickness_mm: f64, deflection_mm: f64, limits: &Limits,
    admission: Option<&mut dyn FnMut(u64, u64) -> om9_core::Result<()>>) -> Result<Brep, EditingError> {
    let deflection = checked_deflection(deflection_mm)?;
    positive(thickness_mm, "shell thickness")?;
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
    snapshot_via_with_admission(deflection, limits, admission, |sink, context, message, capacity| {
        // SAFETY: source and callback context live through this call.
        unsafe {
            om9_occt_shell(
                source.as_ptr(),
                source.len(),
                face_indices.as_ptr(),
                face_indices.len(),
                triangles.map_or(std::ptr::null(), |(triangles, _, _)| triangles.as_ptr().cast()),
                triangles.map_or(0.0, |(_, tolerance, _)| tolerance),
                triangles.map_or(0.0, |(_, _, d)| d),
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

#[cfg(test)]
mod surface_guard_tests {
    use super::*;
    #[test]
    fn native_surface_serialization_budget_rejects_before_snapshot() {
        unsafe extern "C" fn sink(context:*mut c_void,_:*const crate::brep::Snapshot)->c_int {
            // SAFETY: the test passes a live mutable bool as the callback context.
            unsafe {*context.cast::<bool>()=true};0
        }
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
        let face=crate::measurement::surface_to_face(&surface,om9_core::geometry::Transform::identity(),crate::creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
        let source=face.source().as_bytes();let mut called=false;let mut message=[0 as c_char;512];
        // SAFETY: source, message and callback context stay live for this synchronous call.
        let status=unsafe {om9_occt_sew(source.as_ptr(),source.len(),source.as_ptr(),source.len(),0,1,0.001,0.1,1000,1000,4,sink,(&mut called as *mut bool).cast(),message.as_mut_ptr(),512)};
        assert_ne!(status,0);assert!(!called);
    }
}
