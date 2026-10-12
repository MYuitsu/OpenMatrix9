// SPDX-License-Identifier: LGPL-2.1-or-later
//! Live initialized immutable one-allocation buffers are a caller obligation;
//! bounds checks do not prove accessibility. No native handles/allocators escape.
use crate::edge_geometry::{self as model,Edge,Vertex,Geometry};
use crate::support_snapshot_ffi::{JsonSummary,output_valid,copy_json};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Input {pub edges:*const Edge,pub edge_count:usize,pub vertices:*const Vertex,pub vertex_count:usize}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edge_geometry_counts(edges:usize,vertices:usize)->u32{u32::from(!model::counts_allowed(edges,vertices))}
#[unsafe(no_mangle)]
pub extern "C" fn om9_edge_tolerance(value:f64)->u32{u32::from(!model::tolerance_valid(value))}
fn valid<T>(p:*const T,n:usize)->bool{
 if n==0{return p.is_null()||(p as usize)%std::mem::align_of::<T>()==0;}
 !p.is_null()&&(p as usize)%std::mem::align_of::<T>()==0&&n.checked_mul(std::mem::size_of::<T>()).is_some_and(|bytes|bytes<=isize::MAX as usize&&(p as usize).checked_add(bytes).is_some())
}
/// # Safety
/// Live initialized immutable rows; nonoverlapping writable output required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_edge_geometry_json(input:Input,output:*mut u8,capacity:usize)->JsonSummary{
 let error=|status|JsonSummary{status,..Default::default()};
 if !output_valid(output,capacity){return error(1002);}
 if !model::counts_allowed(input.edge_count,input.vertex_count)||!valid(input.edges,input.edge_count)||!valid(input.vertices,input.vertex_count){return error(8000);}
 // SAFETY: guarded envelopes; live initialized immutable storage and
 // nonoverlapping writable output remain caller obligations. Avoid null slices.
 let edges=if input.edge_count==0{&[]}else{unsafe{std::slice::from_raw_parts(input.edges,input.edge_count)}};
 let vertices=if input.vertex_count==0{&[]}else{unsafe{std::slice::from_raw_parts(input.vertices,input.vertex_count)}};
 let geometry=match Geometry::capture(edges,vertices){Ok(v)=>v,Err(_)=>return error(8001)};
 unsafe{copy_json(geometry.json(),output,capacity)}
}
