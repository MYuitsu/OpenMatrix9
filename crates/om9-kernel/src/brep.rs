// SPDX-License-Identifier: LGPL-2.1-or-later
//! Complete native snapshot operations; all returned state is owned by Rust.
use crate::KernelError;
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    geometry::{Mesh, NativeBrep},
};
use std::{
    ffi::{CStr, c_char, c_int, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
};
use uuid::Uuid;

#[repr(C)]
pub(crate) struct Snapshot {
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
    fn om9_occt_brep_from_3dm_snapshot(
        payload: *const u8,
        length: usize,
        tolerance: f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_to_3dm(
        source: *const u8,
        length: usize,
        tolerance: f64,
        max_bytes: usize,
        crc: *mut u32,
        sink: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
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
    fn om9_occt_brep_from_triangles(
        points: *const f64,
        point_count: usize,
        triangles: *const [u32; 3],
        triangle_count: usize,
        tolerance: f64,
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

fn bound(value: u64) -> usize {
    usize::try_from(value)
        .unwrap_or(usize::MAX)
        .min(isize::MAX as usize)
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
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: callback/context and all input buffers live through this synchronous complete
        // operation.
        unsafe {
            om9_occt_brep_snapshot(
                source.as_ptr(),
                source.len(),
                kind,
                dimensions.as_ptr(),
                deflection,
                bound(limits.max_points),
                bound(limits.max_nodes),
                bound(limits.max_bytes),
                sink,
                context,
                message,
                capacity,
            )
        }
    })
}

/// The bounded native budget passed to every snapshot-producing adapter.
pub(crate) struct SnapshotBudget {
    pub(crate) points: usize,
    pub(crate) nodes: usize,
    pub(crate) bytes: usize,
}

pub(crate) fn snapshot_budget(limits: &Limits) -> SnapshotBudget {
    SnapshotBudget {
        points: bound(limits.max_points),
        nodes: bound(limits.max_nodes),
        bytes: bound(limits.max_bytes),
    }
}

/// Run one synchronous native constructor and turn its snapshot into an owned [`Brep`].
///
/// The adapter writes its bounded snapshot through `sink` and returns `0`; any other status is
/// reported as a typed kernel error carrying the adapter's message. A Rust-side refusal during
/// the snapshot copy keeps the adapter's own status non-zero and the copy error wins, so a
/// partially admitted shape is never returned.
pub(crate) fn snapshot_via(
    deflection: f64,
    limits: &Limits,
    invoke: impl FnOnce(
        unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        *mut c_void,
        *mut c_char,
        c_int,
    ) -> c_int,
) -> Result<Brep, KernelError> {
    match snapshot_via_optional(deflection, limits, invoke)? {
        Some(brep) => Ok(brep),
        None => Err(KernelError::InvalidInput("BRep snapshot missing".into())),
    }
}

/// The typed "valid but empty" native status: a boolean whose result has no face. An empty result
/// is a legitimate outcome, not a zero-face shape, so it never becomes a [`Brep`].
pub(crate) const NATIVE_EMPTY: c_int = 3;

/// Like [`snapshot_via`], but `NATIVE_EMPTY` maps to `Ok(None)` and no snapshot is required.
pub(crate) fn snapshot_via_optional(
    deflection: f64,
    limits: &Limits,
    invoke: impl FnOnce(
        unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        *mut c_void,
        *mut c_char,
        c_int,
    ) -> c_int,
) -> Result<Option<Brep>, KernelError> {
    let mut context = Context {
        limits,
        deflection,
        result: None,
    };
    let mut message = [0i8; 512];
    let status = invoke(
        receive,
        (&mut context as *mut Context<'_>).cast(),
        message.as_mut_ptr(),
        512,
    );
    if let Some(result) = context.result {
        return result
            .map(Some)
            .map_err(|error| KernelError::InvalidInput(error.to_string()));
    }
    if status == NATIVE_EMPTY {
        return Ok(None);
    }
    if status != 0 {
        // SAFETY: native adapter always null-terminates a nonempty fixed-size message buffer.
        let text = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        // Status 2 is an OCCT failure after the input was admitted; status 1 is an adapter-level
        // input refusal. Keeping them distinct lets an editing owner report `Kernel` vs `Invalid`.
        return Err(if status == 2 {
            KernelError::Failure(text)
        } else {
            KernelError::InvalidInput(text)
        });
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
/// The sewing tolerance used when a mesh becomes one editable B-Rep. Gem facets share their corner
/// coordinates exactly, so a micron is far below any authored feature and still above OCCT noise.
pub const MESH_BREP_TOLERANCE_MM: f64 = 1.0e-6;
/// Build one closed B-Rep from an indexed triangle mesh.
///
/// The mesh must be a closed manifold: every edge belongs to exactly two triangles. The adapter
/// sews one planar face per triangle into a single shell and promotes it to a solid, so an open or
/// non-manifold mesh is refused instead of being sealed shut.
pub fn from_triangles(
    points: &[[f64; 3]],
    triangles: &[[u32; 3]],
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    if points.len() < 4
        || triangles.len() < 4
        || !deflection_mm.is_finite()
        || deflection_mm <= 0.
        || points.len() as u64 > limits.max_points
        || triangles.len() as u64 > limits.max_nodes
    {
        return Err(KernelError::InvalidInput("mesh BRep input budget".into()));
    }
    if triangles
        .iter()
        .any(|t| t.iter().any(|corner| *corner as usize >= points.len()))
    {
        return Err(KernelError::InvalidInput(
            "mesh BRep corner out of range".into(),
        ));
    }
    snapshot_via(deflection_mm, limits, |sink, context, message, capacity| {
        // SAFETY: both flat arrays are live for the synchronous complete call and the adapter only
        // copies from them.
        unsafe {
            om9_occt_brep_from_triangles(
                points.as_ptr().cast::<f64>(),
                points.len(),
                triangles.as_ptr(),
                triangles.len(),
                MESH_BREP_TOLERANCE_MM,
                deflection_mm,
                bound(limits.max_points),
                bound(limits.max_nodes),
                bound(limits.max_bytes),
                sink,
                context,
                message,
                capacity,
            )
        }
    })
}

/// Build one closed B-Rep from a core [`Mesh`], fan-triangulating its 3- and 4-corner faces.
///
/// The entity keeps the authored mesh as its portable form; this adapter produces the same boundary
/// as OCCT topology for the kernel-side operations.
pub fn from_mesh(mesh: &Mesh, deflection_mm: f64, limits: &Limits) -> Result<Brep, KernelError> {
    mesh.validate(limits)
        .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
    let corners: usize = mesh.faces.iter().map(|face| face.len() - 2).sum();
    let mut triangles = Vec::new();
    triangles
        .try_reserve_exact(corners)
        .map_err(|_| KernelError::InvalidInput("mesh BRep budget".into()))?;
    for face in &mesh.faces {
        for index in 1..face.len() - 1 {
            triangles.push([face[0], face[index], face[index + 1]]);
        }
    }
    from_triangles(&mesh.vertices, &triangles, deflection_mm, limits)
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

struct BytesContext {
    bytes: Vec<u8>,
    limit: usize,
}

unsafe extern "C" fn receive_bytes(context: *mut c_void, bytes: *const u8, length: usize) -> c_int {
    // SAFETY: the callback is synchronous and the context lives for the whole call.
    let context = unsafe { &mut *context.cast::<BytesContext>() };
    if length > context.limit.saturating_sub(context.bytes.len()) {
        return 1;
    }
    if length > 0 {
        if bytes.is_null() {
            return 1;
        }
        // SAFETY: native guarantees a live buffer for this synchronous callback.
        context
            .bytes
            .extend_from_slice(unsafe { std::slice::from_raw_parts(bytes, length) });
    }
    0
}

fn native_message(message: &[c_char; 512]) -> String {
    // SAFETY: the native bridge always NUL-terminates within the provided buffer.
    unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn byte_bound(limits: &Limits) -> usize {
    usize::try_from(limits.max_bytes)
        .unwrap_or(usize::MAX)
        .min(isize::MAX as usize)
}

/// Convert a 3DM `NativeBrep` payload into an owned editable OCCT `Brep`.
/// The native side converts the shape and returns the complete snapshot in one
/// call; the OCCT text is not reparsed a second time.
pub fn from_3dm(
    payload: &[u8],
    tolerance_mm: f64,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    if payload.is_empty()
        || payload.len() as u64 > limits.max_bytes
        || !tolerance_mm.is_finite()
        || tolerance_mm <= 0.
        || !deflection_mm.is_finite()
        || deflection_mm <= 0.
    {
        return Err(KernelError::InvalidInput("3DM BRep input budget".into()));
    }
    let mut context = Context {
        limits,
        deflection: deflection_mm,
        result: None,
    };
    let mut message = [0i8; 512];
    // SAFETY: input and callback context live through this synchronous complete call.
    let status = unsafe {
        om9_occt_brep_from_3dm_snapshot(
            payload.as_ptr(),
            payload.len(),
            tolerance_mm,
            deflection_mm,
            usize::try_from(limits.max_points).unwrap_or(usize::MAX),
            usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
            byte_bound(limits),
            receive,
            (&mut context as *mut Context<'_>).cast(),
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(KernelError::InvalidInput(native_message(&message)));
    }
    let result = context
        .result
        .ok_or_else(|| KernelError::Failure("3DM BRep snapshot missing".into()))?;
    result.map_err(|error| KernelError::InvalidInput(error.to_string()))
}

/// Convert an editable OCCT `Brep` into a 3DM `NativeBrep` payload. The display
/// mesh is derived from the already admitted Brep display, never from native
/// export bytes.
pub fn to_3dm(brep: &Brep, tolerance_mm: f64, limits: &Limits) -> Result<NativeBrep, KernelError> {
    brep.validate(limits)
        .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
    if !tolerance_mm.is_finite() || tolerance_mm <= 0. {
        return Err(KernelError::InvalidInput("BRep export tolerance".into()));
    }
    let mut context = BytesContext {
        bytes: Vec::new(),
        limit: byte_bound(limits),
    };
    let mut crc = 0u32;
    let mut message = [0i8; 512];
    // SAFETY: source, scalar output and callback context live through this synchronous call.
    let status = unsafe {
        om9_occt_brep_to_3dm(
            brep.source().as_ptr(),
            brep.source().len(),
            tolerance_mm,
            byte_bound(limits),
            &mut crc,
            receive_bytes,
            (&mut context as *mut BytesContext).cast(),
            message.as_mut_ptr(),
            512,
        )
    };
    if status != 0 {
        return Err(KernelError::InvalidInput(native_message(&message)));
    }
    if context.bytes.is_empty() {
        return Err(KernelError::Failure("empty ON_Brep payload".into()));
    }
    let faces = brep
        .display
        .triangles
        .iter()
        .map(|triangle| vec![triangle[0], triangle[1], triangle[2]])
        .collect();
    let display = Mesh {
        vertices: brep.display.points.clone(),
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    };
    Ok(NativeBrep {
        display,
        payload: context.bytes,
        crc,
    })
}
