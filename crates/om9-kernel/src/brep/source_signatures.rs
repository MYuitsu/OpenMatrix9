//! Geometric signatures from one OCCT source read, without tessellation.
//!
//! Indices are local to that read: they do not prove correspondence with a
//! previously captured display or persisted UUIDs.
use std::{ffi::{CStr, c_char, c_int, c_void}, panic::{AssertUnwindSafe, catch_unwind}};
use om9_core::{Error, Result, associative::{SubshapeKind, SubshapeSignature}, budget::{Limits, Usage}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceSignature {
    pub kind: SubshapeKind,
    pub source_index: usize,
    pub signature: SubshapeSignature,
}

unsafe extern "C" {
    fn om9_occt_snapshot_signatures(handle: *const c_void, max_nodes: usize,
        sink: unsafe extern "C" fn(*mut c_void, c_int, usize, *const f64) -> c_int, context: *mut c_void) -> c_int;
    fn om9_occt_source_signatures(source: *const u8, length: usize, max_nodes: usize,
        sink: unsafe extern "C" fn(*mut c_void, c_int, usize, *const f64) -> c_int,
        context: *mut c_void, handle_sink: Option<unsafe extern "C" fn(*mut c_void, *const c_void) -> c_int>,
        handle_context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
    fn om9_occt_shape_set_signatures(packet: *const u8, length: usize, max_nodes: usize, max_bytes: usize,
        mapping: unsafe extern "C" fn(*mut c_void, c_int, c_int, u32, u32, c_int, *const u8, usize) -> c_int,
        mapping_context: *mut c_void, sink: unsafe extern "C" fn(*mut c_void, c_int, usize, *const f64) -> c_int,
        context: *mut c_void, message: *mut c_char, capacity: c_int) -> c_int;
}

/// The native handle is valid only during the caller's snapshot callback.
pub(super) unsafe fn snapshot_signatures(handle: *const c_void, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>, canceled: &dyn Fn() -> bool) -> Result<Vec<SourceSignature>> {
    if handle.is_null() { return Err(Error::Invalid("signature snapshot handle")); }
    if canceled() { return Err(Error::Canceled); }
    let mut state = Collector { limits, admit, canceled, values: Vec::new(), expected: None, next: [0; 3], error: None };
    let status = unsafe { om9_occt_snapshot_signatures(handle, super::bound(limits.max_nodes), receive,
        (&mut state as *mut Collector<'_>).cast()) };
    if let Some(error) = state.error { return Err(error); }
    if status != 0 || state.expected != Some(state.values.len()) { return Err(Error::Invalid("snapshot signatures")); }
    Ok(state.values)
}

struct MappingVisitor<'a> {
    visit: &'a mut dyn FnMut(c_int, u32, u32, c_int) -> Result<()>,
    error: Option<Error>,
}
unsafe extern "C" fn visit_mapping(context: *mut c_void, event: c_int, kind: c_int, old: u32, new: u32,
    orientation: c_int, _data: *const u8, len: usize) -> c_int {
    let state = unsafe { &mut *context.cast::<MappingVisitor<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        if event != 0 || len != 0 { return Err(Error::Invalid("shape set signature mapping event")); }
        (state.visit)(kind, old, new, orientation)
    })).unwrap_or(Err(Error::Invalid("shape set signature mapping panic")));
    if let Err(error) = result { state.error = Some(error); 1 } else { 0 }
}

struct HandleVisitor<'a> {
    visit: &'a mut dyn FnMut(*const c_void) -> Result<()>,
    error: Option<Error>,
}
unsafe extern "C" fn visit_handle(context: *mut c_void, handle: *const c_void) -> c_int {
    let state = unsafe { &mut *context.cast::<HandleVisitor<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| (state.visit)(handle)))
        .unwrap_or(Err(Error::Invalid("signature topology callback panic")));
    if let Err(error) = result { state.error = Some(error); 1 } else { 0 }
}

struct Collector<'a> {
    limits: &'a Limits,
    admit: &'a mut dyn FnMut(u64, u64) -> Result<()>,
    canceled: &'a dyn Fn() -> bool,
    values: Vec<SourceSignature>,
    expected: Option<usize>,
    next: [usize; 3],
    error: Option<Error>,
}

unsafe extern "C" fn receive(context: *mut c_void, kind: c_int, index: usize, data: *const f64) -> c_int {
    let state = unsafe { &mut *context.cast::<Collector<'_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        if (state.canceled)() { return Err(Error::Canceled); }
        if kind == -2 { return Ok(()); }
        if kind == -1 {
            if state.expected.is_some() || index as u64 > state.limits.max_nodes { return Err(Error::Invalid("signature count")); }
            let planned = Usage::array::<SourceSignature>(index)?;
            planned.check(state.limits)?;
            (state.admit)(0, planned.bytes)?;
            state.values.try_reserve_exact(index).map_err(|_| Error::Budget("source signatures"))?;
            let actual = Usage::array::<SourceSignature>(state.values.capacity())?;
            actual.check(state.limits)?;
            (state.admit)(actual.bytes, 0)?;
            state.expected = Some(index);
            return Ok(());
        }
        let k = usize::try_from(kind).map_err(|_| Error::Invalid("signature kind"))?;
        if k >= 3 || state.expected.is_none() || data.is_null() || state.values.len() >= state.expected.unwrap()
            || state.next[k] != index { return Err(Error::Invalid("signature record")); }
        let values = unsafe { std::slice::from_raw_parts(data, 7) };
        let signature = SubshapeSignature { start: [values[0], values[1], values[2]],
            end: [values[3], values[4], values[5]], measure_mm: values[6] };
        signature.validate()?;
        let kind = [SubshapeKind::Face, SubshapeKind::Edge, SubshapeKind::Vertex][k];
        state.values.push(SourceSignature { kind, source_index: index, signature });
        state.next[k] += 1;
        Ok(())
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => { state.error = Some(error); 1 }
        Err(_) => { state.error = Some(Error::Invalid("signature callback panic")); 1 }
    }
}

/// Read exact-source geometry without generating a viewport mesh. The caller
/// must separately charge the live source and keep the output reservation for
/// as long as the returned allocation is held. Native OCCT parser, maps and
/// geometric integration are not covered by the Rust allocation admission.
/// Cancellation is polled between shapes, not inside parsing or integration.
pub fn source_signatures(source: &str, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>,
    canceled: impl Fn() -> bool) -> Result<Vec<SourceSignature>> {
    source_signatures_with_handle(source, limits, &mut admit, &canceled, None)
}

pub(super) fn source_signatures_with_handle(source: &str, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    canceled: &dyn Fn() -> bool, visit: Option<&mut dyn FnMut(*const c_void) -> Result<()>>) -> Result<Vec<SourceSignature>> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes { return Err(Error::Invalid("signature source")); }
    let mut state = Collector { limits, admit, canceled, values: Vec::new(),
        expected: None, next: [0; 3], error: None };
    let mut visitor = visit.map(|visit| HandleVisitor { visit, error: None });
    let (handle_sink, handle_context) = match visitor.as_mut() {
        Some(visitor) => (Some(visit_handle as unsafe extern "C" fn(*mut c_void, *const c_void) -> c_int),
            (visitor as *mut HandleVisitor<'_>).cast()),
        None => (None, std::ptr::null_mut()),
    };
    let mut message = [0i8; 512];
    let status = unsafe { om9_occt_source_signatures(source.as_ptr(), source.len(), super::bound(limits.max_nodes),
        receive, (&mut state as *mut Collector<'_>).cast(), handle_sink, handle_context, message.as_mut_ptr(), message.len() as c_int) };
    if let Some(error) = state.error { return Err(error); }
    if let Some(error) = visitor.and_then(|visitor| visitor.error) { return Err(error); }
    if status != 0 { return Err(Error::Io(unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned())); }
    if state.expected != Some(state.values.len()) { return Err(Error::Invalid("incomplete source signatures")); }
    Ok(state.values)
}

/// The caller supplies a private shape-set packet and owns the mapping state.
pub(super) fn shape_set_signatures(packet: &[u8], limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    canceled: &dyn Fn() -> bool, mapping: &mut dyn FnMut(c_int, u32, u32, c_int) -> Result<()>) -> Result<Vec<SourceSignature>> {
    if packet.is_empty() || packet.len() as u64 > limits.max_bytes { return Err(Error::Invalid("shape set signatures")); }
    if canceled() { return Err(Error::Canceled); }
    let mut state = Collector { limits, admit, canceled, values: Vec::new(), expected: None, next: [0; 3], error: None };
    let mut visitor = MappingVisitor { visit: mapping, error: None };
    let mut message = [0i8; 512];
    let status = unsafe { om9_occt_shape_set_signatures(packet.as_ptr(), packet.len(), super::bound(limits.max_nodes),
        super::bound(limits.max_bytes), visit_mapping, (&mut visitor as *mut MappingVisitor<'_>).cast(), receive,
        (&mut state as *mut Collector<'_>).cast(), message.as_mut_ptr(), message.len() as c_int) };
    if let Some(error) = state.error { return Err(error); }
    if let Some(error) = visitor.error { return Err(error); }
    if status != 0 { return Err(Error::Io(unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned())); }
    if state.expected != Some(state.values.len()) { return Err(Error::Invalid("incomplete shape set signatures")); }
    Ok(state.values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn snapshot_signatures_match_created_brep_topology() {
        let limits = Limits::default();
        let dimensions = [2., 3., 4.];
        let (brep, signatures) = super::super::snapshot_via_signatures(0.1, &limits, |_, _| Ok(()), &|| false,
            |sink, context, message, capacity| unsafe {
                super::super::om9_occt_brep_snapshot(std::ptr::null(), 0, 0, dimensions.as_ptr(), 0.1, 0.3,
                    super::super::bound(limits.max_points), super::super::bound(limits.max_nodes), super::super::bound(limits.max_bytes),
                    sink, context, message, capacity)
            }).unwrap();
        assert_eq!(signatures.len(), 26);
        let vertex_start = brep.topology().faces.len() + brep.topology().edges.len();
        for (index, value) in signatures[vertex_start..].iter().enumerate() {
            assert_eq!(value.kind, SubshapeKind::Vertex);
            assert_eq!(value.source_index, index);
            assert_eq!(value.signature.start, brep.display().points[brep.display().vertices[index] as usize]);
        }
        let area: f64 = signatures.iter().filter(|value| value.kind == SubshapeKind::Face)
            .map(|value| value.signature.measure_mm).sum();
        assert!((area - 52.).abs() < 1e-8);
    }

    #[test]
    fn source_signatures_survive_remesh_and_measure_box() {
        let limits = Limits::default();
        let original = super::super::box_brep([2., 3., 4.], 0.3, &limits).unwrap();
        let remeshed = super::super::read_brep(original.source(), 0.02, &limits).unwrap();
        let read = |source: &str| source_signatures(source, &limits, |_, _| Ok(()), || false).unwrap();
        let first = read(original.source());
        assert_eq!(first, read(remeshed.source()));
        let faces: Vec<_> = first.iter().filter(|s| s.kind == SubshapeKind::Face).collect();
        let edges: Vec<_> = first.iter().filter(|s| s.kind == SubshapeKind::Edge).collect();
        assert_eq!((faces.len(), edges.len(), first.iter().filter(|s| s.kind == SubshapeKind::Vertex).count()), (6, 12, 8));
        assert!((faces.iter().map(|s| s.signature.measure_mm).sum::<f64>() - 52.).abs() < 1e-8);
        assert!((edges.iter().map(|s| s.signature.measure_mm).sum::<f64>() - 36.).abs() < 1e-8);
    }

    #[test]
    fn source_signatures_respect_admission_and_cancellation() {
        let limits = Limits::default();
        let original = super::super::box_brep([2., 3., 4.], 0.3, &limits).unwrap();
        assert!(matches!(source_signatures(original.source(), &limits, |_, _| Err(Error::Budget("test")), || false), Err(Error::Budget("test"))));
        let calls = Cell::new(0);
        let result = source_signatures(original.source(), &limits, |_, _| Ok(()), || {
            calls.set(calls.get() + 1);
            calls.get() > 4
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert!(calls.get() > 4);
        assert!(source_signatures("not a brep", &limits, |_, _| Ok(()), || false).is_err());
    }

    #[test]
    fn source_signatures_measure_sphere_without_viewport_mesh() {
        let limits = Limits::default();
        let sphere = super::super::sphere_brep(5., 0.4, &limits).unwrap();
        let source = sphere.source().to_owned();
        drop(sphere);
        let values = source_signatures(&source, &limits, |_, _| Ok(()), || false).unwrap();
        let faces: Vec<_> = values.iter().filter(|value| value.kind == SubshapeKind::Face).collect();
        assert_eq!(faces.len(), 1);
        assert!((faces[0].signature.measure_mm - 100. * std::f64::consts::PI).abs() < 1e-7);
    }
}
