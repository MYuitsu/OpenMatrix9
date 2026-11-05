// SPDX-License-Identifier: LGPL-2.1-or-later
//! Complete native snapshot operations; all returned state is owned by Rust.
use crate::KernelError;
use om9_core::{
    brep::{Brep, BrepDisplay},
    budget::Limits,
    geometry::{Mesh, NativeBrep},
};
use std::{
    cell::{Cell, RefCell},
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
        angular_deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_from_3dm(
        payload: *const u8, length: usize, tolerance: f64, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int,
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
        angular_deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        context: *mut c_void,
        message: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn om9_occt_brep_snapshot_grouped(
        source: *const u8, source_len: usize, deflection: f64, angular_deflection: f64,
        max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int, context: *mut c_void,
        shell_sink: unsafe extern "C" fn(*mut c_void, *const u32, usize, *const EdgeNodeBinding, usize, f64) -> c_int, shell_context: *mut c_void,
        include_edge_bindings: c_int, optimize_export_sphere: c_int,
        message: *mut c_char, capacity: c_int,
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
struct Context<'a, 'callback> {
    limits: &'a Limits,
    deflection: f64,
    result: Option<om9_core::Result<Brep>>,
    grouped_bytes: Option<&'a Cell<u64>>,
    admission: Option<&'a SnapshotAdmission<'callback>>,
}
type SnapshotAdmission<'a> = RefCell<&'a mut dyn FnMut(u64, u64) -> om9_core::Result<()>>;

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

/// Minimum owned storage implied by the native lengths, before copying any
/// array. Native buffers and transient allocator/validation scratch are not
/// covered by this lower bound.
fn admit_snapshot_copy(s: &Snapshot, limits: &Limits) -> om9_core::Result<om9_core::budget::Usage> {
    use om9_core::{Error, budget::Usage};
    let identities = s.faces_count.checked_add(s.edges_count)
        .and_then(|n| n.checked_add(s.vertices_len)).ok_or(Error::Overflow("BRep identities"))?;
    if identities as u128 > u128::from(limits.max_nodes) || s.points_len as u128 > u128::from(limits.max_points) {
        return Err(Error::Budget("BRep topology"));
    }
    let mut usage = Usage::default();
    for part in [
        Usage::array::<u8>(s.source_len)?,
        Usage::array::<[f64; 3]>(s.points_len)?,
        Usage::array::<[u32; 4]>(s.triangles_len)?,
        Usage::array::<[u32; 3]>(s.segments_len)?,
        Usage::array::<u32>(s.vertices_len)?,
        Usage::array::<[u32; 2]>(s.face_edges_len)?,
        Usage::array::<Uuid>(identities)?,
        Usage::array::<usize>(2)?,
        Usage::array::<String>(1)?,
    ] {
        if part.bytes > isize::MAX as u64 { return Err(Error::Budget("native BRep array")); }
        usage.add(part, limits)?;
    }
    Ok(usage)
}

fn admit_group_copy(count: usize, edge_count: usize, other: u64, limit: u64) -> om9_core::Result<u64> {
    use om9_core::{Error, budget::Usage};
    let groups = Usage::array::<u32>(count)?.bytes;
    let edges = Usage::array::<EdgeNodeBinding>(edge_count)?.bytes;
    if groups > isize::MAX as u64 || edges > isize::MAX as u64 { return Err(Error::Budget("BRep edge bindings")); }
    let remaining = limit.checked_sub(other).ok_or(Error::Budget("BRep edge bindings"))?;
    if groups.checked_add(edges).ok_or(Error::Overflow("BRep edge bindings"))? > remaining {
        return Err(Error::Budget("BRep edge bindings"));
    }
    Ok(remaining)
}

#[cfg(test)]
mod snapshot_copy_admission_tests {
    use super::*;
    fn lengths_only() -> Snapshot {
        Snapshot {
            source: std::ptr::null(), source_len: 1,
            points: std::ptr::null(), points_len: 2,
            triangles: std::ptr::null(), triangles_len: 2,
            segments: std::ptr::null(), segments_len: 0,
            vertices: std::ptr::null(), vertices_len: 0,
            face_edges: std::ptr::null(), face_edges_len: 0,
            faces_count: 1, edges_count: 0,
        }
    }

    #[test]
    fn aggregate_is_checked_before_any_native_array_is_copied() {
        let snapshot = lengths_only();
        let required = 1 + 2 * std::mem::size_of::<[f64; 3]>()
            + 2 * std::mem::size_of::<[u32; 4]>() + std::mem::size_of::<Uuid>()
            + 2 * std::mem::size_of::<usize>() + std::mem::size_of::<String>();
        let limits = Limits { max_bytes: required as u64 - 1, ..Limits::default() };
        assert!(admit_snapshot_copy(&snapshot, &limits).is_err());
        let mut context = Context { limits: &limits, deflection: 0.1, result: None, grouped_bytes: None, admission: None };
        // All arrays are null; a budget refusal demonstrates that none is read.
        let status = unsafe { receive((&mut context as *mut Context<'_, '_>).cast(), &snapshot) };
        assert_ne!(status, 0);
        assert!(matches!(context.result, Some(Err(om9_core::Error::Budget("bytes")))));
        let enough = Limits { max_bytes: required as u64, ..limits };
        assert!(admit_snapshot_copy(&snapshot, &enough).is_ok());
    }

    #[test]
    fn snapshot_length_overflow_is_rejected_without_allocation() {
        let mut snapshot = lengths_only();
        snapshot.triangles_len = usize::MAX;
        assert!(matches!(admit_snapshot_copy(&snapshot, &Limits::default()), Err(om9_core::Error::Overflow(_))));
    }

    #[test]
    fn grouped_and_snapshot_copy_minima_share_one_limit_in_either_order() {
        let snapshot = lengths_only();
        let required = (1 + 2 * std::mem::size_of::<[f64; 3]>()
            + 2 * std::mem::size_of::<[u32; 4]>() + std::mem::size_of::<Uuid>()
            + 2 * std::mem::size_of::<usize>() + std::mem::size_of::<String>()) as u64;
        let limits = Limits { max_bytes: required + 3, ..Limits::default() };
        let account = Cell::new(4);
        let mut context = Context { limits: &limits, deflection: 0.1, result: None, grouped_bytes: Some(&account), admission: None };
        let status = unsafe { receive((&mut context as *mut Context<'_, '_>).cast(), &snapshot) };
        assert_ne!(status, 0);
        assert!(matches!(context.result, Some(Err(om9_core::Error::Budget("bytes")))));
        assert_eq!(account.get(), 4);
        assert!(admit_group_copy(1, 0, required, limits.max_bytes).is_err());
        assert!(admit_group_copy(1, 0, required, required + 4).is_ok());
    }

    #[test]
    fn successful_snapshot_adds_actual_storage_to_existing_group_charge() {
        let limits = Limits::default();
        let body = box_brep([10.; 3], 0.1, &limits).unwrap();
        let d = &body.display;
        let snapshot = Snapshot {
            source: body.source().as_ptr(), source_len: body.source().len(),
            points: d.points.as_ptr(), points_len: d.points.len(),
            triangles: d.triangles.as_ptr(), triangles_len: d.triangles.len(),
            segments: d.segments.as_ptr(), segments_len: d.segments.len(),
            vertices: d.vertices.as_ptr(), vertices_len: d.vertices.len(),
            face_edges: d.face_edges.as_ptr(), face_edges_len: d.face_edges.len(),
            faces_count: d.faces.len(), edges_count: d.edges.len(),
        };
        let account = Cell::new(32);
        let mut context = Context { limits: &limits, deflection: 0.1, result: None, grouped_bytes: Some(&account), admission: None };
        let status = unsafe { receive((&mut context as *mut Context<'_, '_>).cast(), &snapshot) };
        assert_eq!(status, 0);
        let copied = context.result.unwrap().unwrap();
        assert_eq!(account.get(), 32 + copied.storage_usage(&limits).unwrap().bytes);
        assert_eq!(copied.display.points, body.display.points);
        assert_eq!(copied.display.triangles, body.display.triangles);
        let failed_after_callback = snapshot_via_optional(0.1, &limits, |sink, context, _, _| {
            assert_eq!(unsafe { sink(context, &snapshot) }, 0);
            2
        });
        assert!(matches!(failed_after_callback, Err(KernelError::Failure(_))));
    }
}
unsafe extern "C" fn receive(context: *mut c_void, snapshot: *const Snapshot) -> c_int {
    // SAFETY: both pointers are live only for this synchronous invocation, never retained.
    let context = unsafe { &mut *context.cast::<Context<'_, '_>>() };
    if context.result.is_some() {
        context.result = Some(Err(om9_core::Error::Invalid("duplicate BRep snapshot")));
        return 1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| -> om9_core::Result<Brep> {
        let s = unsafe { &*snapshot };
        let other = context.grouped_bytes.map_or(0, Cell::get);
        let effective = context.limits.remaining_after(om9_core::budget::Usage { bytes: other, ..Default::default() })?;
        let l = &effective;
        let copy = admit_snapshot_copy(s, l)?;
        if let Some(admit) = context.admission {
            let ids = s.faces_count.checked_add(s.edges_count).and_then(|n| n.checked_add(s.vertices_len))
                .ok_or(om9_core::Error::Overflow("BRep identities"))?;
            let scratch = om9_core::budget::Usage::array::<Uuid>(ids)?.bytes;
            let overhead = std::mem::size_of::<BrepDisplay>() as u64 + 2 * std::mem::size_of::<usize>() as u64;
            let pending = copy.bytes.checked_add(scratch).and_then(|n| n.checked_add(overhead))
                .ok_or(om9_core::Error::Overflow("BRep copy admission"))?;
            (admit.borrow_mut())(other, pending)?;
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
        let brep = Brep::from_native(source, display, l)?;
        if let Some(account) = context.grouped_bytes {
            let bytes = brep.storage_usage(l)?.bytes;
            account.set(other.checked_add(bytes).ok_or(om9_core::Error::Overflow("grouped BRep copy"))?);
        }
        if let Some(admit) = context.admission {
            (admit.borrow_mut())(other.checked_add(brep.storage_usage(l)?.bytes)
                .ok_or(om9_core::Error::Overflow("BRep copy admission"))?, 0)?;
        }
        Ok(brep)
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
    snapshot_with_angle(source, kind, dimensions, deflection, 0.3, limits)
}

fn snapshot_with_angle(
    source: &str,
    kind: i32,
    dimensions: [f64; 3],
    deflection: f64,
    angular_deflection: f64,
    limits: &Limits,
) -> Result<Brep, KernelError> {
    if !angular_deflection.is_finite() || angular_deflection <= 0.0 || angular_deflection > std::f64::consts::PI {
        return Err(KernelError::InvalidInput("BRep angular deflection".into()));
    }
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
                angular_deflection,
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
    snapshot_via_optional_accounted(deflection, limits, None, None, invoke)
}

fn snapshot_via_optional_accounted(
    deflection: f64,
    limits: &Limits,
    grouped_bytes: Option<&Cell<u64>>,
    admission: Option<&SnapshotAdmission<'_>>,
    invoke: impl FnOnce(
        unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int,
        *mut c_void, *mut c_char, c_int,
    ) -> c_int,
) -> Result<Option<Brep>, KernelError> {
    let mut context = Context {
        limits,
        deflection,
        result: None,
        grouped_bytes,
        admission,
    };
    let mut message = [0i8; 512];
    let status = invoke(
        receive,
        (&mut context as *mut Context<'_, '_>).cast(),
        message.as_mut_ptr(),
        512,
    );
    if let Some(Err(error)) = &context.result {
        return Err(KernelError::InvalidInput(error.to_string()));
    }
    if status == NATIVE_EMPTY && context.result.is_none() {
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
    if let Some(Ok(brep)) = context.result { return Ok(Some(brep)); }
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
    read_brep_with_angle(source, deflection_mm, 0.3, limits)
}

/// Tessellate canonical OCCT source with an explicit angular deflection in radians.
pub fn read_brep_with_angle(source: &str, deflection_mm: f64, angular_deflection: f64, limits: &Limits) -> Result<Brep, KernelError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes {
        return Err(KernelError::InvalidInput("BRep source budget".into()));
    }
    snapshot_with_angle(source, 0, [0.; 3], deflection_mm, angular_deflection, limits)
}

/// Export-only, snapshot-local face-to-shell ownership from the same OCCT shape as the mesh.
/// Group zero denotes a face not owned by a shell; IDs are not persistent topology identities.
pub fn read_brep_with_shell_groups(source: &str, deflection_mm: f64, angular_deflection: f64, limits: &Limits) -> Result<(Brep, Vec<u32>), KernelError> {
    let (brep, owners, _, _) = read_brep_grouped(source, deflection_mm, angular_deflection, limits, false, false, None)?;
    Ok((brep, owners))
}

/// Snapshot-local edge-node association; a missing curve parameter is explicit.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EdgeNodeBinding { pub point: u32, pub edge: u32, vertex: u32, reserved: u32, parameter: f64 }
impl EdgeNodeBinding {
    pub fn parameter(&self) -> Option<f64> { self.parameter.is_finite().then_some(self.parameter) }
    /// Snapshot-local canonical endpoint identity, when OCCT's endpoint and node agree.
    pub fn vertex(&self) -> Option<u32> { (self.vertex != u32::MAX).then_some(self.vertex) }
    fn same_source_site(&self, other: &Self) -> bool {
        if let (Some(a), Some(b)) = (self.vertex(), other.vertex()) { return a == b; }
        self.edge == other.edge && self.parameter().zip(other.parameter()).is_some_and(|(a, b)| a == b)
    }
}

/// Edge-node identities owned by one immutable tessellation snapshot.
/// Comparisons only read entries from this set. Callers must derive both indices from this set.
#[derive(Debug)]
pub struct EdgeBindingSet(Vec<EdgeNodeBinding>);
impl EdgeBindingSet {
    pub fn bindings(&self) -> &[EdgeNodeBinding] { &self.0 }
    /// Compare source topology only. The caller must separately validate transformed world distance.
    pub fn same_source_site(&self, first: usize, second: usize) -> Option<bool> {
        Some(self.0.get(first)?.same_source_site(self.0.get(second)?))
    }
}

/// Export-only source, mesh and bindings are kept together so a comparison cannot use a different
/// snapshot's mesh. Sampled viewport edge polylines are intentionally absent.
#[derive(Debug)]
pub struct EdgeTopologySnapshot {
    brep: Brep,
    shell_groups: Vec<u32>,
    bindings: EdgeBindingSet,
    max_source_tolerance: f64,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TopologySite { Vertex(u32), EdgeParameter(u32, u64) }
impl EdgeTopologySnapshot {
    pub fn brep(&self) -> &Brep { &self.brep }
    /// Consume this snapshot after any topology mapping is complete. The
    /// returned transient display owns its buffers; bindings are discarded.
    /// Refuse a shared display instead of allocating an unreserved clone.
    pub fn into_display(self) -> om9_core::Result<om9_core::brep::BrepDisplay> {
        std::sync::Arc::try_unwrap(self.brep.display).map_err(|_| om9_core::Error::Busy)
    }
    pub fn shell_groups(&self) -> &[u32] { &self.shell_groups }
    pub fn bindings(&self) -> &EdgeBindingSet { &self.bindings }
    /// Largest OCCT face, edge or vertex tolerance in source coordinates.
    /// This metadata is not a measured surface or exported-mesh error.
    pub fn max_source_tolerance(&self) -> f64 { self.max_source_tolerance }
    /// Inventory the owned Rust snapshot and buffer capacities. This excludes
    /// native meshing transients, allocator overhead and caller-owned scratch;
    /// callers must reserve those separately before constructing the snapshot.
    pub fn retained_usage(&self, limits: &Limits) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::budget::Usage;
        let mut usage = Usage::array::<Self>(1)?;
        usage.add(self.brep.storage_usage(limits)?, limits)?;
        usage.add(Usage::array::<u32>(self.shell_groups.capacity())?, limits)?;
        usage.add(Usage::array::<EdgeNodeBinding>(self.bindings.0.capacity())?, limits)?;
        Ok(usage)
    }
    /// A topology match also has to fit the permitted separation in exported world units.
    pub fn may_weld(&self, first: usize, second: usize, placement: om9_core::geometry::Transform, max_distance: f64) -> om9_core::Result<bool> {
        if !max_distance.is_finite() || max_distance < 0.0 { return Err(om9_core::Error::Invalid("STL weld distance")); }
        placement.validate()?;
        let a = self.bindings.0.get(first).ok_or(om9_core::Error::Invalid("BRep binding index"))?;
        let b = self.bindings.0.get(second).ok_or(om9_core::Error::Invalid("BRep binding index"))?;
        let p = *self.brep.display.points.get(a.point as usize).ok_or(om9_core::Error::Invalid("BRep binding point"))?;
        let q = *self.brep.display.points.get(b.point as usize).ok_or(om9_core::Error::Invalid("BRep binding point"))?;
        weld_candidate(a, b, placement, p, q, max_distance)
    }

    /// Conservative experimental remapping. Unbound nodes stay separate; topology identities
    /// and a bound to the final representative are both required for every merged component.
    pub fn topology_remap(&self, placement: om9_core::geometry::Transform, max_distance: f64) -> om9_core::Result<Vec<u32>> {
        self.topology_remap_with_check(placement, max_distance, || Ok(()))
    }

    /// Minimum simultaneous storage of the three indexed remap arrays. Tree
    /// maps, allocator over-allocation and caller-owned data are not included.
    pub fn topology_remap_minimum_usage(&self) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::budget::Usage;
        let count = self.brep.display.points.len();
        let limits = Limits { max_bytes: u64::MAX, ..Limits::default() };
        let mut usage = Usage::array::<u32>(count)?;
        usage.add(Usage::array::<f64>(count)?, &limits)?;
        usage.add(Usage::array::<u32>(count)?, &limits)?;
        Ok(usage)
    }

    /// Include conservative BTree storage for at most one entry per eligible
    /// binding. Duplicate keys and unused shell bindings may reduce actual use.
    pub fn topology_remap_estimated_usage(&self) -> om9_core::Result<om9_core::budget::Usage> {
        self.topology_remap_estimated_usage_with_check(|| Ok(()))
    }

    fn topology_remap_estimated_usage_with_check(&self, mut check: impl FnMut() -> om9_core::Result<()>) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::budget::Usage;
        let limits = Limits { max_bytes: u64::MAX, ..Limits::default() };
        let mut usage = self.topology_remap_minimum_usage()?;
        let mut vertex_count = 0usize;
        let mut site_count = 0usize;
        for (index, binding) in self.bindings.0.iter().enumerate() {
            if index % 1024 == 0 { check()?; }
            if binding.vertex().is_some() { vertex_count += 1; site_count += 1; }
            else if binding.parameter().is_some() { site_count += 1; }
        }
        if site_count != 0 { usage.add(Usage::btree::<(u32, TopologySite), u32>(site_count)?, &limits)?; }
        if vertex_count != 0 { usage.add(Usage::btree::<u32, u32>(vertex_count)?, &limits)?; }
        Ok(usage)
    }

    /// Admit the snapshot, remap arrays and estimated lookup storage before
    /// allocating them. The caller must deduct other live owners from `limits`.
    pub fn topology_remap_with_budget(
        &self, placement: om9_core::geometry::Transform, max_distance: f64, limits: &Limits,
        mut check: impl FnMut() -> om9_core::Result<()>,
    ) -> om9_core::Result<Vec<u32>> {
        check()?;
        let mut retained = self.retained_usage(limits)?;
        retained.add(self.topology_remap_estimated_usage_with_check(&mut check)?, limits)?;
        self.topology_remap_with_check(placement, max_distance, check)
    }

    pub fn topology_remap_with_check(&self, placement: om9_core::geometry::Transform, max_distance: f64, mut check: impl FnMut() -> om9_core::Result<()>) -> om9_core::Result<Vec<u32>> {
        use om9_core::Error;
        use std::collections::BTreeMap;
        check()?;
        if !max_distance.is_finite() || max_distance < 0.0 { return Err(Error::Invalid("STL weld distance")); }
        placement.validate()?;
        let count = self.brep.display.points.len();
        let mut parent = Vec::new();
        parent.try_reserve_exact(count).map_err(|_| Error::Budget("STL topology remap"))?;
        for i in 0..count { parent.push(u32::try_from(i).map_err(|_| Error::Overflow("STL topology points"))?); }
        let mut radius = Vec::new();
        radius.try_reserve_exact(count).map_err(|_| Error::Budget("STL topology remap"))?;
        radius.resize(count, 0.0_f64);
        let mut shell = Vec::new();
        shell.try_reserve_exact(count).map_err(|_| Error::Budget("STL topology remap"))?;
        shell.resize(count, u32::MAX);
        for (index, [a, b, c, face]) in self.brep.display.triangles.iter().enumerate() {
            if index % 1024 == 0 { check()?; }
            let group = *self.shell_groups.get(*face as usize).ok_or(Error::Invalid("STL shell face"))?;
            for point in [a, b, c] {
                let owner = shell.get_mut(*point as usize).ok_or(Error::Invalid("STL shell point"))?;
                if *owner != u32::MAX && *owner != group { return Err(Error::Invalid("STL point belongs to multiple shells")); }
                *owner = group;
            }
        }
        fn root(parent: &mut [u32], mut index: u32) -> u32 {
            while parent[index as usize] != index {
                let next = parent[index as usize];
                parent[index as usize] = parent[next as usize];
                index = next;
            }
            index
        }
        let mut sites = BTreeMap::<(u32, TopologySite), u32>::new();
        let mut vertex_of_point = BTreeMap::<u32, u32>::new();
        for (index, binding) in self.bindings.0.iter().enumerate() {
            if index % 1024 == 0 { check()?; }
            let point = binding.point;
            let group = *shell.get(point as usize).ok_or(Error::Invalid("STL binding point"))?;
            if group == u32::MAX { continue; }
            let key = if let Some(vertex) = binding.vertex() {
                if vertex_of_point.insert(point, vertex).is_some_and(|old| old != vertex) {
                    return Err(Error::Invalid("STL conflicting source vertices"));
                }
                TopologySite::Vertex(vertex)
            } else if let Some(parameter) = binding.parameter() {
                TopologySite::EdgeParameter(binding.edge, if parameter == 0.0 { 0 } else { parameter.to_bits() })
            } else { continue };
            if let Some(previous) = sites.get(&(group, key)) {
                let a = root(&mut parent, *previous);
                let b = root(&mut parent, point);
                if a != b {
                    let (a, b) = if a < b { (a, b) } else { (b, a) };
                    let distance = world_separation(placement, self.brep.display.points[a as usize], self.brep.display.points[b as usize])?;
                    let bound = radius[b as usize] + distance;
                    if !bound.is_finite() || bound > max_distance { return Err(Error::Invalid("STL topology weld exceeds distance")); }
                    parent[b as usize] = a;
                    radius[a as usize] = radius[a as usize].max(bound);
                }
            } else {
                sites.insert((group, key), point);
            }
        }
        for i in 0..count { if i % 1024 == 0 { check()?; } parent[i] = root(&mut parent, i as u32); }
        check()?;
        Ok(parent)
    }
}

fn weld_candidate(a: &EdgeNodeBinding, b: &EdgeNodeBinding, placement: om9_core::geometry::Transform,
                  p: [f64; 3], q: [f64; 3], max_distance: f64) -> om9_core::Result<bool> {
    if !a.same_source_site(b) { return Ok(false); }
    Ok(world_separation(placement, p, q)? <= max_distance)
}

fn world_separation(placement: om9_core::geometry::Transform, a: [f64; 3], b: [f64; 3]) -> om9_core::Result<f64> {
    placement.validate()?;
    if a.iter().chain(b.iter()).any(|v| !v.is_finite()) { return Err(om9_core::Error::Invalid("STL weld point")); }
    // Translation cancels analytically; subtracting transformed large coordinates would lose small gaps.
    let delta = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let v: [f64; 3] = std::array::from_fn(|i| (0..3).map(|j| placement.0[i][j] * delta[j]).sum());
    let distance = v[0].hypot(v[1]).hypot(v[2]);
    if !distance.is_finite() { return Err(om9_core::Error::Invalid("STL weld distance")); }
    Ok(distance)
}

#[cfg(test)]
mod edge_site_tests {
    use super::{EdgeNodeBinding, world_separation, weld_candidate};
    fn binding(edge: u32, vertex: Option<u32>, parameter: Option<f64>) -> EdgeNodeBinding {
        EdgeNodeBinding { point: 0, edge, vertex: vertex.unwrap_or(u32::MAX), reserved: 0, parameter: parameter.unwrap_or(f64::NAN) }
    }
    #[test]
    fn export_snapshot_omits_display_edges_but_preserves_mesh_and_bindings() {
        use om9_core::budget::Limits;
        let limits = Limits::default();
        let body = super::box_brep([10., 7., 3.], 0.1, &limits).unwrap();
        let (display, owners) = super::read_brep_with_shell_groups(body.source(), 0.1, 0.3, &limits).unwrap();
        let export = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        println!("snapshot_counts display_points={} export_points={} display_segments={} export_segments={} triangles={}",
            display.display.points.len(), export.brep.display.points.len(), display.display.segments.len(),
            export.brep.display.segments.len(), export.brep.display.triangles.len());
        assert!(!display.display.segments.is_empty());
        assert!(export.brep.display.segments.is_empty());
        assert!(export.brep.display.points.len() < display.display.points.len());
        assert_eq!(export.brep.display.triangles, display.display.triangles);
        let mesh_points = export.brep.display.triangles.iter().flat_map(|triangle| triangle[..3].iter())
            .copied().max().unwrap() as usize + 1;
        assert_eq!(&export.brep.display.points[..mesh_points], &display.display.points[..mesh_points]);
        assert_eq!(export.shell_groups(), owners);
        assert!(!export.bindings().bindings().is_empty());
        assert!(export.bindings().bindings().iter().all(|binding| (binding.point as usize) < mesh_points));
    }
    #[test]
    fn consuming_display_moves_unique_buffers_and_refuses_shared_snapshot() {
        use om9_core::{budget::Limits, Error};
        let limits = Limits::default();
        let body = super::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
        let snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let original_points = snapshot.brep.display.points.as_ptr();
        let display = snapshot.into_display().unwrap();
        assert_eq!(display.points.as_ptr(), original_points, "unique buffer must be moved");
        let shared = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let alias = shared.brep.display.clone();
        let original_points = alias.points.as_ptr();
        assert!(matches!(shared.into_display(), Err(Error::Busy)));
        assert_eq!(alias.points.as_ptr(), original_points);
        assert!(!alias.triangles.is_empty());
    }
    #[test]
    fn remap_rejects_when_snapshot_and_minimum_arrays_do_not_fit_together() {
        use om9_core::{budget::Limits, geometry::Transform, Error};
        let limits = Limits::default();
        let body = super::box_brep([10.; 3], 0.1, &limits).unwrap();
        let snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let retained = snapshot.retained_usage(&limits).unwrap();
        let arrays = snapshot.topology_remap_minimum_usage().unwrap();
        assert_eq!(arrays.bytes, snapshot.brep.display.points.len() as u64 * 16);
        let estimate = snapshot.topology_remap_estimated_usage().unwrap();
        assert!(estimate.bytes > arrays.bytes, "fixture must contain lookup entries");
        let arrays_only = Limits { max_bytes: retained.bytes + arrays.bytes, ..limits };
        assert!(matches!(snapshot.topology_remap_with_budget(Transform::identity(), 1e-6, &arrays_only, || Ok(())), Err(Error::Budget(_))));
        let required = retained.bytes + estimate.bytes;
        let low = Limits { max_bytes: required - 1, ..limits };
        assert!(matches!(snapshot.topology_remap_with_budget(Transform::identity(), 1e-6, &low, || Ok(())), Err(Error::Budget(_))));
        let exact = Limits { max_bytes: required, ..limits };
        let admitted = snapshot.topology_remap_with_budget(Transform::identity(), 1e-6, &exact, || Ok(())).unwrap();
        assert_eq!(admitted, snapshot.topology_remap(Transform::identity(), 1e-6).unwrap());
        assert!(matches!(snapshot.topology_remap_with_budget(Transform::identity(), 1e-6, &low, || Err(Error::Canceled)), Err(Error::Canceled)));
    }
    #[test]
    fn remap_budget_estimate_checks_cancellation_during_binding_scan() {
        use om9_core::{budget::Limits, geometry::Transform, Error};
        let limits = Limits::default();
        let body = super::box_brep([10.; 3], 0.1, &limits).unwrap();
        let mut snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let binding = snapshot.bindings.0[0];
        snapshot.bindings.0.resize(2050, binding);
        let mut checks = 0;
        let result = snapshot.topology_remap_with_budget(Transform::identity(), 1e-6, &limits, || {
            checks += 1;
            if checks == 3 { Err(Error::Canceled) } else { Ok(()) }
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(checks, 3);
    }
    #[test]
    fn remap_without_bindings_does_not_charge_empty_lookup_trees() {
        use om9_core::budget::Limits;
        let limits = Limits::default();
        let body = super::box_brep([10.; 3], 0.1, &limits).unwrap();
        let mut snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        snapshot.bindings.0.clear();
        assert_eq!(snapshot.topology_remap_estimated_usage().unwrap(), snapshot.topology_remap_minimum_usage().unwrap());
    }
    #[test]
    fn snapshot_inventory_counts_retained_capacity_and_rejects_tight_limit() {
        use om9_core::budget::Limits;
        let limits = Limits::default();
        let body = super::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
        let mut snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let before = snapshot.retained_usage(&limits).unwrap();
        let capacities = (snapshot.brep.display.points.capacity(), snapshot.shell_groups.capacity(), snapshot.bindings.0.capacity());
        std::sync::Arc::make_mut(&mut snapshot.brep.display).points.reserve(1000);
        snapshot.shell_groups.reserve(1000);
        snapshot.bindings.0.reserve(1000);
        let after = snapshot.retained_usage(&limits).unwrap();
        let delta = (snapshot.brep.display.points.capacity() - capacities.0) * std::mem::size_of::<[f64; 3]>()
            + (snapshot.shell_groups.capacity() - capacities.1) * std::mem::size_of::<u32>()
            + (snapshot.bindings.0.capacity() - capacities.2) * std::mem::size_of::<EdgeNodeBinding>();
        assert!(delta > 0);
        assert_eq!(after.bytes, before.bytes + delta as u64);
        assert_eq!(after.points, before.points);
        assert_eq!(after.nodes, before.nodes);
        let tight = Limits { max_bytes: after.bytes - 1, ..limits };
        assert!(matches!(snapshot.retained_usage(&tight), Err(om9_core::Error::Budget(_))));
        assert_eq!(snapshot.retained_usage(&limits).unwrap(), after);
    }
    #[test]
    fn source_site_requires_verified_endpoint_or_exact_known_edge_parameter() {
        let v = binding(1, Some(7), Some(0.));
        assert!(v.same_source_site(&binding(2, Some(7), Some(5.))));
        assert!(!v.same_source_site(&binding(1, Some(8), Some(0.))));
        assert!(binding(1, None, Some(0.5)).same_source_site(&binding(1, None, Some(0.5))));
        assert!(!binding(1, None, Some(0.5)).same_source_site(&binding(1, None, Some(0.500000000001))));
        assert!(!binding(1, None, None).same_source_site(&binding(1, None, None)));
        assert!(!binding(1, None, Some(0.5)).same_source_site(&binding(2, None, Some(0.5))));
    }
    #[test]
    fn weld_distance_respects_nonuniform_scale_mirror_and_large_translation() {
        use om9_core::geometry::Transform;
        let a = [0., 0., 0.]; let b = [1e-7, 0., 0.];
        assert!(world_separation(Transform::identity(), a, b).unwrap() < 5e-7);
        let mut placement = Transform::scale([-10., 1., 1.]).unwrap();
        placement.0[0][3] = 1e20;
        assert!(world_separation(placement, a, b).unwrap() > 5e-7);
        let source = binding(1, None, Some(0.5));
        assert!(weld_candidate(&source, &source, Transform::identity(), a, b, 5e-7).unwrap());
        assert!(!weld_candidate(&source, &source, placement, a, b, 5e-7).unwrap());
        assert!(!weld_candidate(&source, &binding(1, None, Some(0.6)), Transform::identity(), a, b, 5e-7).unwrap());
        assert!(world_separation(placement, [f64::NAN, 0., 0.], b).is_err());
    }
    #[test]
    fn topology_remap_refuses_transitive_distance_growth() {
        use om9_core::{budget::Limits, geometry::Transform};
        let limits = Limits::default();
        let body = super::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
        let mut snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let vertex = snapshot.bindings.0.iter().find_map(|binding| binding.vertex()).unwrap();
        let points: Vec<_> = snapshot.bindings.0.iter().filter(|binding| binding.vertex() == Some(vertex))
            .map(|binding| binding.point).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
        assert!(points.len() >= 3, "box vertex must be represented on three faces");
        for (point, x) in points.iter().zip([0., 0.9e-6, 1.8e-6]) {
            std::sync::Arc::make_mut(&mut snapshot.brep.display).points[*point as usize] = [x, 0., 0.];
        }
        assert!(matches!(snapshot.topology_remap(Transform::identity(), 1e-6), Err(om9_core::Error::Invalid("STL topology weld exceeds distance"))));
    }
    #[test]
    fn topology_remap_propagates_cancellation() {
        use om9_core::{budget::Limits, geometry::Transform, Error};
        let limits = Limits::default();
        let body = super::box_brep([1., 1., 1.], 0.1, &limits).unwrap();
        let snapshot = super::read_brep_with_shell_groups_and_edges(body.source(), 0.1, 0.3, &limits).unwrap();
        let mut checks = 0;
        let result = snapshot.topology_remap_with_check(Transform::identity(), 1e-6, || {
            checks += 1;
            if checks >= 3 { Err(Error::Canceled) } else { Ok(()) }
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(checks, 3);
    }
}

/// Also return snapshot-local edge-node bindings for export-seam investigation.
pub fn read_brep_with_shell_groups_and_edges(source: &str, deflection_mm: f64, angular_deflection: f64, limits: &Limits) -> Result<EdgeTopologySnapshot, KernelError> {
    read_brep_with_shell_groups_and_edges_admitted(source, deflection_mm, angular_deflection, limits, |_, _| Ok(()))
}

/// Admit owned Rust copies before allocation and reconcile capacity afterward.
/// Native meshing work precedes these callbacks and is not bounded by them.
pub fn read_brep_with_shell_groups_and_edges_admitted(source: &str, deflection_mm: f64,
    angular_deflection: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<EdgeTopologySnapshot, KernelError> {
    let admission: SnapshotAdmission<'_> = RefCell::new(&mut admit);
    let (brep, owners, bindings, max_source_tolerance) = read_brep_grouped(source, deflection_mm, angular_deflection, limits, true, true, Some(&admission))?;
    Ok(EdgeTopologySnapshot { brep, shell_groups: owners, bindings: EdgeBindingSet(bindings), max_source_tolerance })
}

fn read_brep_grouped(source: &str, deflection_mm: f64, angular_deflection: f64, limits: &Limits, include_edge_bindings: bool, optimize_export_sphere: bool,
    admission: Option<&SnapshotAdmission<'_>>) -> Result<(Brep, Vec<u32>, Vec<EdgeNodeBinding>, f64), KernelError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes
        || !angular_deflection.is_finite() || angular_deflection <= 0.0 || angular_deflection > std::f64::consts::PI {
        return Err(KernelError::InvalidInput("grouped BRep snapshot input".into()));
    }
    struct Groups<'a, 'callback> { limit: u64, nodes: u64, account: &'a Cell<u64>, admission: Option<&'a SnapshotAdmission<'callback>>, result: Option<om9_core::Result<(Vec<u32>, Vec<EdgeNodeBinding>, f64)>> }
    unsafe extern "C" fn receive_groups(context: *mut c_void, data: *const u32, count: usize, edges: *const EdgeNodeBinding, edge_count: usize, tolerance: f64) -> c_int {
        let context = unsafe { &mut *context.cast::<Groups<'_, '_>>() };
        if context.result.is_some() {
            context.result = Some(Err(om9_core::Error::Invalid("duplicate BRep shell groups")));
            return 1;
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            if !tolerance.is_finite() || tolerance < 0.0 { return Err(om9_core::Error::Invalid("BRep source tolerance")); }
            if count as u64 > context.nodes { return Err(om9_core::Error::Budget("BRep shell groups")); }
            let other = context.account.get();
            let remaining = admit_group_copy(count, edge_count, other, context.limit)?;
            if let Some(admit) = context.admission {
                let pending = om9_core::budget::Usage::array::<u32>(count)?.bytes.checked_add(
                    om9_core::budget::Usage::array::<EdgeNodeBinding>(edge_count)?.bytes)
                    .ok_or(om9_core::Error::Overflow("BRep group admission"))?;
                (admit.borrow_mut())(other, pending)?;
            }
            let owners = unsafe { copy_array(data, count, remaining)? };
            let bindings = unsafe { copy_array(edges, edge_count, remaining)? };
            let actual = om9_core::budget::Usage::array::<u32>(owners.capacity())?.bytes
                .checked_add(om9_core::budget::Usage::array::<EdgeNodeBinding>(bindings.capacity())?.bytes)
                .ok_or(om9_core::Error::Overflow("BRep edge bindings"))?;
            if actual > remaining { return Err(om9_core::Error::Budget("BRep edge bindings")); }
            context.account.set(other.checked_add(actual).ok_or(om9_core::Error::Overflow("grouped BRep copy"))?);
            if let Some(admit) = context.admission { (admit.borrow_mut())(context.account.get(), 0)?; }
            Ok((owners, bindings, tolerance))
        }));
        context.result = Some(result.unwrap_or(Err(om9_core::Error::Invalid("BRep group adapter panic"))));
        i32::from(context.result.as_ref().is_none_or(|r| r.is_err()))
    }
    let account = Cell::new(0);
    let mut groups = Groups { limit: limits.max_bytes, nodes: limits.max_nodes, account: &account, admission, result: None };
    let budget = snapshot_budget(limits);
    let brep = snapshot_via_optional_accounted(deflection_mm, limits, Some(&account), admission, |sink, context, message, capacity| unsafe {
        om9_occt_brep_snapshot_grouped(source.as_ptr(), source.len(), deflection_mm, angular_deflection,
            budget.points, budget.nodes, budget.bytes, sink, context, receive_groups,
            (&mut groups as *mut Groups<'_, '_>).cast(), i32::from(include_edge_bindings), i32::from(optimize_export_sphere), message, capacity)
    })?.ok_or_else(|| KernelError::InvalidInput("BRep snapshot missing".into()))?;
    let (owners, edge_bindings, tolerance) = groups.result.ok_or_else(|| KernelError::InvalidInput("BRep shell groups missing".into()))?
        .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
    if owners.len() != brep.display.faces.len() {
        return Err(KernelError::InvalidInput("BRep shell group count".into()));
    }
    if edge_bindings.iter().any(|binding| binding.point as usize >= brep.display.points.len() || binding.edge as usize >= brep.display.edges.len() || binding.vertex().is_some_and(|vertex| vertex as usize >= brep.display.vertices.len()) || binding.reserved != 0 || binding.parameter.is_infinite()) {
        return Err(KernelError::InvalidInput("BRep edge binding index".into()));
    }
    Ok((brep, owners, edge_bindings, tolerance))
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

pub(crate) struct BytesContext {
    pub(crate) bytes: Vec<u8>,
    pub(crate) limit: usize,
}

/// Operation-local hook for owned source bytes. The native operation itself
/// runs synchronously before returning these bytes and needs separate admission.
pub(crate) struct AdmittedBytesContext<'a> {
    pub(crate) bytes: Vec<u8>,
    limit: usize,
    admit: &'a mut dyn FnMut(u64, u64) -> om9_core::Result<()>,
    pub(crate) error: Option<om9_core::Error>,
}
impl<'a> AdmittedBytesContext<'a> {
    pub(crate) fn new(limit: usize, admit: &'a mut dyn FnMut(u64, u64) -> om9_core::Result<()>) -> Self {
        Self { bytes: Vec::new(), limit, admit, error: None }
    }
}
pub(crate) unsafe extern "C" fn receive_admitted_bytes(context: *mut c_void, bytes: *const u8, length: usize) -> c_int {
    // SAFETY: native invokes synchronously with the live operation context and
    // contiguous byte span. Neither pointer is retained across the callback.
    let context = unsafe { &mut *context.cast::<AdmittedBytesContext<'_>>() };
    if context.error.is_some() { return 1; }
    let result = catch_unwind(AssertUnwindSafe(|| -> om9_core::Result<()> {
        let needed = context.bytes.len().checked_add(length).ok_or(om9_core::Error::Overflow("native source copy"))?;
        if needed > context.limit || (length != 0 && bytes.is_null()) { return Err(om9_core::Error::Budget("native source copy")); }
        let pending = if needed > context.bytes.capacity() { needed as u64 } else { 0 };
        (context.admit)(context.bytes.capacity() as u64, pending)?;
        context.bytes.try_reserve_exact(length).map_err(|_| om9_core::Error::Budget("native source copy"))?;
        if context.bytes.capacity() > context.limit { return Err(om9_core::Error::Budget("native source copy")); }
        (context.admit)(context.bytes.capacity() as u64, 0)?;
        if length != 0 { context.bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(bytes, length) }); }
        Ok(())
    })).unwrap_or(Err(om9_core::Error::Invalid("native source admission panic")));
    if let Err(error) = result { context.error = Some(error); return 1; }
    0
}

#[cfg(test)]
mod staged_source_admission_tests {
    use super::*;
    #[test]
    fn source_copy_refuses_before_allocating_and_catches_callback_panic() {
        for panic in [false, true] {
            let mut admit = |_, _| -> om9_core::Result<()> {
                if panic { panic!("injected source callback panic"); }
                Err(om9_core::Error::Budget("injected source copy refusal"))
            };
            let mut context = AdmittedBytesContext::new(1024, &mut admit);
            let status = unsafe { receive_admitted_bytes((&mut context as *mut AdmittedBytesContext<'_>).cast(), b"source".as_ptr(), 6) };
            assert_eq!(status, 1);
            assert_eq!(context.bytes.capacity(), 0);
            assert!(context.error.is_some());
        }
    }
    #[test]
    fn grouped_snapshot_admission_preserves_mesh_and_stops_owned_copy() {
        let limits = Limits::default();
        let body = box_brep([10.; 3], 0.1, &limits).unwrap();
        let reference = read_brep_with_shell_groups_and_edges(body.source(), 0.05, 0.3, &limits).unwrap();
        let mut planned = false;
        let mut allocated = false;
        let snapshot = read_brep_with_shell_groups_and_edges_admitted(body.source(), 0.05, 0.3, &limits, |live, pending| {
            planned |= pending > 0;
            allocated |= live > 0 && pending == 0;
            Ok(())
        }).unwrap();
        assert!(planned && allocated);
        assert_eq!(snapshot.brep().display.points, reference.brep().display.points);
        assert_eq!(snapshot.brep().display.triangles, reference.brep().display.triangles);
        assert_eq!(snapshot.shell_groups(), reference.shell_groups());
        let mut seen = 0;
        let refused = read_brep_with_shell_groups_and_edges_admitted(body.source(), 0.05, 0.3, &limits, |_, pending| {
            seen += 1;
            assert!(pending > 0);
            Err(om9_core::Error::Budget("injected snapshot copy refusal"))
        });
        assert!(refused.is_err());
        assert!(seen > 0);
    }
}

pub(crate) unsafe extern "C" fn receive_bytes(context: *mut c_void, bytes: *const u8, length: usize) -> c_int {
    // SAFETY: the callback is synchronous and the context lives for the whole call.
    let context = unsafe { &mut *context.cast::<BytesContext>() };
    if length > context.limit.saturating_sub(context.bytes.len()) {
        return 1;
    }
    if length > 0 {
        if bytes.is_null() {
            return 1;
        }
        if context.bytes.try_reserve(length).is_err() { return 1; }
        // SAFETY: native guarantees a live buffer for this synchronous callback.
        context
            .bytes
            .extend_from_slice(unsafe { std::slice::from_raw_parts(bytes, length) });
    }
    0
}

pub(crate) fn native_message(message: &[c_char; 512]) -> String {
    // SAFETY: the native bridge always NUL-terminates within the provided buffer.
    unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn byte_bound(limits: &Limits) -> usize {
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
    from_3dm_with_angle(payload, tolerance_mm, deflection_mm, 0.3, limits)
}

/// Convert a retained 3DM payload to canonical OCCT source without generating a display mesh.
/// The caller must validate and tessellate that source for its own purpose.
pub fn from_3dm_source(payload: &[u8], tolerance_mm: f64, limits: &Limits) -> Result<String, KernelError> {
    from_3dm_source_admitted(payload, tolerance_mm, limits, |_, _| Ok(()))
}
pub fn from_3dm_source_admitted(payload: &[u8], tolerance_mm: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> om9_core::Result<()>) -> Result<String, KernelError> {
    if payload.is_empty() || payload.len() as u64 > limits.max_bytes || !tolerance_mm.is_finite() || tolerance_mm <= 0. {
        return Err(KernelError::InvalidInput("3DM BRep input budget".into()));
    }
    let mut context = AdmittedBytesContext::new(byte_bound(limits), &mut admit);
    let mut message = [0i8; 512];
    // SAFETY: native invokes the callback synchronously while both input and context are alive.
    let status = unsafe { om9_occt_brep_from_3dm(payload.as_ptr(), payload.len(), tolerance_mm,
        byte_bound(limits), receive_admitted_bytes, (&mut context as *mut AdmittedBytesContext<'_>).cast(), message.as_mut_ptr(), 512) };
    if let Some(error) = context.error { return Err(KernelError::InvalidInput(error.to_string())); }
    if status != 0 { return Err(KernelError::InvalidInput(native_message(&message))); }
    if context.bytes.is_empty() { return Err(KernelError::InvalidInput("3DM BRep source missing".into())); }
    String::from_utf8(context.bytes).map_err(|_| KernelError::InvalidInput("3DM BRep source encoding".into()))
}

/// Convert a retained 3DM BRep using an explicit angular deflection in radians.
pub fn from_3dm_with_angle(
    payload: &[u8], tolerance_mm: f64, deflection_mm: f64,
    angular_deflection: f64, limits: &Limits,
) -> Result<Brep, KernelError> {
    if payload.is_empty()
        || payload.len() as u64 > limits.max_bytes
        || !tolerance_mm.is_finite()
        || tolerance_mm <= 0.
        || !deflection_mm.is_finite()
        || deflection_mm <= 0.
        || !angular_deflection.is_finite()
        || angular_deflection <= 0.0
        || angular_deflection > std::f64::consts::PI
    {
        return Err(KernelError::InvalidInput("3DM BRep input budget".into()));
    }
    let mut context = Context {
        limits,
        deflection: deflection_mm,
        result: None,
        grouped_bytes: None,
        admission: None,
    };
    let mut message = [0i8; 512];
    // SAFETY: input and callback context live through this synchronous complete call.
    let status = unsafe {
        om9_occt_brep_from_3dm_snapshot(
            payload.as_ptr(),
            payload.len(),
            tolerance_mm,
            deflection_mm,
            angular_deflection,
            usize::try_from(limits.max_points).unwrap_or(usize::MAX),
            usize::try_from(limits.max_nodes).unwrap_or(usize::MAX),
            byte_bound(limits),
            receive,
            (&mut context as *mut Context<'_, '_>).cast(),
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
        payload: context.bytes.into(),
        crc,
    })
}
