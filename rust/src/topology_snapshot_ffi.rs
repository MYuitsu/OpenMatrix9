//! Synchronous borrowed value buffers. Caller supplies initialized, aligned,
//! readable immutable rows for the declared counts; output is writable u32
//! storage of capacity, nonoverlapping with inputs. All storage remains live
//! for the call. No pointers are retained or released. Checks cannot establish
//! accessibility/lifetime of an arbitrary nonnull pointer. Native exceptions
//! stay in C++; release profile aborts on panic/OOM rather than unwinding the ABI.
use crate::topology_snapshot::*;
use std::mem::{align_of,size_of};

#[repr(C)]
#[derive(Clone,Copy,Debug,Default)]
pub struct Summary {
    pub status:u32,
    pub edge_manifold:u32,
    pub edge_closed:u32,
    pub faces:u32,
    pub loops:u32,
    pub edge_uses:u32,
    pub edge_identities:u32,
    pub boundary_components:u32,
    pub degenerate_uses:u32,
    pub vertex_manifold:u32,
    pub face_oriented:u32,
    pub vertex_count:u32,
    pub boundary_vertex_count:u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_topology_counts(faces:usize,loops:usize,uses:usize)->u32 {
    u32::from(!counts_allowed(faces,loops,uses))
}
/// Scalar preflight before native pcurve endpoint evaluation.
#[unsafe(no_mangle)]
pub extern "C" fn om9_topology_domain(a:f64,b:f64)->u32 {
    u32::from(!domain_valid(a,b))
}
fn valid<T>(pointer:*const T,count:usize)->bool {
    count == 0 || (!pointer.is_null() && (pointer as usize).is_multiple_of(align_of::<T>()) &&
                   count.checked_mul(size_of::<T>()).is_some_and(|bytes| bytes <= isize::MAX as usize && (pointer as usize).checked_add(bytes).is_some()))
}
unsafe fn rows<'a,T>(pointer:*const T,count:usize)->&'a[T] {
    if count == 0 { &[] } else {
        // SAFETY: validated envelope and caller's initialized/live buffer contract.
        unsafe{std::slice::from_raw_parts(pointer,count)}
    }
}

/// Rejects atomically: output storage is untouched unless the whole snapshot
/// passes. Requires the live-buffer contract documented at the module boundary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_topology_inspect(
    faces:*const Face,face_count:usize,loops:*const Loop,loop_count:usize,
    uses:*const Use,use_count:usize,components:*mut u32,capacity:usize,
)->Summary {
    let error=|status|Summary{status,..Default::default()};
    if !counts_allowed(face_count,loop_count,use_count) {return error(1);}
    if !valid(faces,face_count) || !valid(loops,loop_count) || !valid(uses,use_count) ||
       capacity < use_count || !valid(components,use_count) {return error(2);}
    // SAFETY: guarded envelopes, caller provides live immutable initialized rows.
    let result=Snapshot::capture(unsafe{rows(faces,face_count)},unsafe{rows(loops,loop_count)},unsafe{rows(uses,use_count)});
    let s=match result {Ok(s)=>s,Err(e)=>return error(10+e as u32)};
    let complex=crate::surface_complex::observe(&s);
    if use_count != 0 {
        // SAFETY: caller provides writable nonoverlapping output storage.
        let output=unsafe{std::slice::from_raw_parts_mut(components,use_count)};
        output.fill(0);
        for (id,group) in s.boundary_components.iter().enumerate() {
            for &index in group {output[index]=(id+1) as u32;}
        }
    }
    Summary{status:0,edge_manifold:u32::from(s.edge_manifold),edge_closed:u32::from(s.edge_closed),
            faces:s.faces.len() as u32,loops:s.loops.len() as u32,edge_uses:s.uses.len() as u32,
            edge_identities:s.edge_identity_count as u32,boundary_components:s.boundary_components.len() as u32,
            degenerate_uses:s.degenerate_use_count as u32,vertex_manifold:u32::from(complex.vertex_manifold),face_oriented:u32::from(complex.face_oriented),vertex_count:complex.vertex_count as u32,boundary_vertex_count:complex.boundary_vertex_count as u32}
}
