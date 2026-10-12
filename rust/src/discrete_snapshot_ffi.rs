// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native callers supply initialized immutable one-allocation buffers, alive
//! for the synchronous call. Bounds checks cannot prove storage validity.
use crate::support_snapshot_ffi::{JsonSummary,output_valid,copy_json};
use crate::discrete_snapshot::{self as model,Geometry};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Input {pub representation:u32,pub reserved:u32,pub points:*const [f64;3],pub point_count:usize,pub faces:*const [u64;3],pub face_count:usize}
#[unsafe(no_mangle)]
pub extern "C" fn om9_discrete_counts(kind:u32,points:usize,faces:usize)->u32{u32::from(!model::counts_allowed(kind,points,faces))}
fn valid<T>(p:*const T,n:usize)->bool {
    if n==0{return p.is_null()||(p as usize)%std::mem::align_of::<T>()==0;}
    !p.is_null()&&(p as usize)%std::mem::align_of::<T>()==0
        &&n.checked_mul(std::mem::size_of::<T>()).is_some_and(|bytes|bytes<=isize::MAX as usize&&(p as usize).checked_add(bytes).is_some())
}
/// # Safety
/// Live initialized immutable inputs and nonoverlapping writable output required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_discrete_json(input:Input,output:*mut u8,capacity:usize)->JsonSummary {
    let error=|status|JsonSummary{status,..Default::default()};
    if !output_valid(output,capacity){return error(1002);}
    if input.reserved!=0||!model::counts_allowed(input.representation,input.point_count,input.face_count)
        ||!valid(input.points,input.point_count)||!valid(input.faces,input.face_count){return error(7000);}
    // SAFETY: envelope validated; initialized storage/lifetime/immutability
    // remain caller obligations. Never form a null zero-length slice.
    let points=if input.point_count==0{&[]}else{unsafe{std::slice::from_raw_parts(input.points,input.point_count)}};
    let faces=if input.face_count==0{&[]}else{unsafe{std::slice::from_raw_parts(input.faces,input.face_count)}};
    let owned=match Geometry::capture(input.representation,points,faces){Ok(v)=>v,Err(_)=>return error(7001)};
    unsafe{copy_json(owned.json(),output,capacity)}
}
