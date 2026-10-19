// SPDX-License-Identifier: LGPL-2.1-or-later
//! Complete native snapshot operations; all returned state is owned by Rust.
use crate::KernelError;
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
};
use std::{
    ffi::{c_char, c_int, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
};
use uuid::Uuid;

#[repr(C)]
struct Snapshot {
    source: *const u8,
    source_len: usize,
    points: *const [f64; 3],
    points_len: usize,
    triangles: *const [u32; 4],
    triangles_len: usize,
    segments: *const [u32; 3],
    segments_len: usize,
    vertices: *const u32,
    vertices_len: usize,
    face_edges: *const [u32; 2],
    face_edges_len: usize,
    faces_count: usize,
    edges_count: usize,
}
unsafe extern "C" {
    fn om9_occt_brep_volume(source: *const u8, length: usize, volume: *mut f64) -> c_int;
    fn om9_occt_brep_snapshot(
        source: *const u8,
        source_len: usize,
        kind: c_int,
        dimensions: *const f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
}
struct Context<'a> {
    limits: &'a Limits,
    deflection: f64,
    result: Option<om9_core::Result<Brep>>,
}
unsafe fn copy_array<T: Copy>(
    pointer: *const T,
    count: usize,
    limit: u64,
) -> om9_core::Result<Vec<T>> {
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(om9_core::Error::Overflow("native BRep array"))?;
    if bytes as u64 > limit || bytes > isize::MAX as usize {
        return Err(om9_core::Error::Budget("native BRep array"));
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    if pointer.is_null() {
        return Err(om9_core::Error::Invalid("native BRep pointer"));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(count)
        .map_err(|_| om9_core::Error::Budget("native BRep copy"))?;
    // SAFETY: the synchronous native adapter provides live contiguous arrays with these lengths.
    out.extend_from_slice(unsafe { std::slice::from_raw_parts(pointer, count) });
    Ok(out)
}
unsafe extern "C" fn receive(context: *mut c_void, snapshot: *const Snapshot) -> c_int {
    // SAFETY: both pointers are live only for this synchronous invocation, never retained.
    let context = unsafe { &mut *context.cast::<Context<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> om9_core::Result<Brep> {
        let s = unsafe { &*snapshot };
        let l = context.limits;
        let count = s
            .faces_count
            .checked_add(s.edges_count)
            .and_then(|n| n.checked_add(s.vertices_len))
            .ok_or(om9_core::Error::Overflow("BRep identities"))?;
        if count as u64 > l.max_nodes || s.points_len as u64 > l.max_points {
            return Err(om9_core::Error::Budget("BRep topology"));
        }
        let source = String::from_utf8(unsafe { copy_array(s.source, s.source_len, l.max_bytes)? })
            .map_err(|_| om9_core::Error::Invalid("BRep source encoding"))?;
        let ids = |n: usize| -> om9_core::Result<Vec<Uuid>> {
            let mut keys = Vec::new();
            keys.try_reserve_exact(n)
                .map_err(|_| om9_core::Error::Budget("BRep identities"))?;
            keys.extend((0..n).map(|_| Uuid::new_v4()));
            Ok(keys)
        };
        let display = BrepDisplay {
            source_sha256: [0; 32],
            deflection_mm: context.deflection,
            points: unsafe { copy_array(s.points, s.points_len, l.max_bytes)? },
            triangles: unsafe { copy_array(s.triangles, s.triangles_len, l.max_bytes)? },
            segments: unsafe { copy_array(s.segments, s.segments_len, l.max_bytes)? },
            vertices: unsafe { copy_array(s.vertices, s.vertices_len, l.max_bytes)? },
            face_edges: unsafe { copy_array(s.face_edges, s.face_edges_len, l.max_bytes)? },
            faces: ids(s.faces_count)?,
            edges: ids(s.edges_count)?,
            vertex_ids: ids(s.vertices_len)?,
        };
        Brep::from_native(source, display, l)
    }));
    context.result = Some(result.unwrap_or(Err(om9_core::Error::Invalid("BRep adapter panic"))));
    i32::from(context.result.as_ref().is_none_or(|r| r.is_err()))
}
fn snapshot(
    source: &str,
    kind: i32,
    dimensions: [f64; 3],
    deflection: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    let mut context = Context {
        limits,
        deflection,
        result: None,
    };
    let mut message = [0i8; 512];
    let bound = |value: u64| {
        usize::try_from(value)
            .unwrap_or(usize::MAX)
            .min(isize::MAX as usize)
    };
    // SAFETY: callback/context and all input buffers live through this synchronous complete operation.
    let status = unsafe {
        om9_occt_brep_snapshot(
            source.as_ptr(),
            source.len(),
            kind,
            dimensions.as_ptr(),
            deflection,
            bound(limits.max_points),
            bound(limits.max_nodes),
            bound(limits.max_bytes),
            receive,
            (&mut context as *mut Context<'_>).cast(),
            message.as_mut_ptr(),
            512,
        )
    };
    if let Some(result) = context.result {
        return result.map_err(|error| KernelError::InvalidInput(error.to_string()));
    }
    if status != 0 {
        // SAFETY: native adapter always null-terminates a nonempty fixed-size message buffer.
        let text = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        return Err(KernelError::InvalidInput(text));
    }
    Err(KernelError::InvalidInput("BRep snapshot missing".into()))
}
pub fn box_brep(
    dimensions: [f64; 3],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    snapshot("", 0, dimensions, deflection_mm, limits)
}
pub fn sphere_brep(
    radius_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    snapshot("", 1, [radius_mm, 0., 0.], deflection_mm, limits)
}
/// Rebuild from canonical OCCT source. New topology UUIDs require a new geometry publication.
pub fn read_brep(source: &str, deflection_mm: f64, limits: &Limits) -> Result<Brep, KernelError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes {
        return Err(KernelError::InvalidInput("BRep source budget".into()));
    }
    snapshot(source, 0, [0.; 3], deflection_mm, limits)
}
/// Measures canonical source, never deriving volume from the display mesh or its bounding box.
pub fn source_volume(shape: &Brep, limits: &Limits) -> Result<f64, KernelError> {
    shape
        .validate(limits)
        .map_err(|e| KernelError::InvalidInput(e.to_string()))?;
    let mut volume = 0.;
    // SAFETY: source and scalar output live through the complete call; native errors are caught.
    let status =
        unsafe { om9_occt_brep_volume(shape.source().as_ptr(), shape.source().len(), &mut volume) };
    if status == 0 {
        Ok(volume)
    } else {
        Err(KernelError::Failure("invalid canonical BRep source".into()))
    }
}
