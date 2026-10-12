// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::cage::*;
use std::alloc::{GlobalAlloc,Layout,System};
use std::cell::Cell;
thread_local!{static TRACK:Cell<bool>=const{Cell::new(false)};static ALLOCS:Cell<usize>=const{Cell::new(0)};}
struct Counting;
unsafe impl GlobalAlloc for Counting{
    unsafe fn alloc(&self,l:Layout)->*mut u8{TRACK.try_with(|t|{if t.get(){ALLOCS.with(|n|n.set(n.get()+1));}}).ok();unsafe{System.alloc(l)}}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{TRACK.try_with(|t|{if t.get(){ALLOCS.with(|n|n.set(n.get()+1));}}).ok();unsafe{System.realloc(p,l,n)}}
}
#[global_allocator]static ALLOCATOR:Counting=Counting;
fn measured<T>(f:impl FnOnce()->T)->(T,usize){ALLOCS.with(|n|n.set(0));TRACK.with(|t|t.set(true));let r=f();TRACK.with(|t|t.set(false));(r,ALLOCS.with(Cell::get))}
fn cage()->Cage{Cage::r#box([2;3],[1;3],[0.;3],[1.;3]).unwrap().0}
fn borrowed(c:&Cage)->BorrowedCage<'_>{BorrowedCage{counts:c.counts,degrees:c.degrees,knots:std::array::from_fn(|a|c.knots[a].as_slice()),points:&c.points,weights:&c.weights}}
fn descriptor(c:&Cage)->Om9CageDescriptor{Om9CageDescriptor{counts:c.counts,degrees:c.degrees,knots:std::array::from_fn(|a|c.knots[a].as_ptr()),
    knot_lengths:std::array::from_fn(|a|c.knots[a].len()),points:c.points.as_ptr().cast(),point_count:c.points.len(),weights:c.weights.as_ptr(),weight_count:c.weights.len(),
    world_to_parameter:IDENTITY,region:0,local_min:[0.;3],local_max:[1.;3],falloff:0.}}
#[test]
fn borrowed_valid_rational_and_imported_counts_need_no_heap_copy(){
    let mut c=cage();c.weights[7]=3.;assert_eq!(measured(||borrowed(&c).validate()),(Ok(()),0));
    c.counts=[129,2,2];c.knots[0]=vec![0.,0.];c.knots[0].extend((1..128).map(|n|n as f64/128.));c.knots[0].extend([1.,1.]);
    c.points=vec![[0.;3];516];c.weights=vec![1.;516];assert_eq!(measured(||borrowed(&c).validate()),(Ok(()),0));
    // Generated boxes retain their separate128 creation limit.
    assert!(Cage::r#box(c.counts,c.degrees,[0.;3],[1.;3]).is_err());
}
#[test]
fn owned_constructor_rejects_bad_basis_without_allocating_or_writing_handle(){unsafe{
    for which in 0..4{let mut c=cage();match which{0=>c.weights[0]=0.,1=>c.points[0][0]=f64::NAN,2=>c.knots[0][2]=0.,_=>c.knots[1][3]=f64::INFINITY};
        let d=descriptor(&c);let mut out=std::ptr::null_mut();let (status,allocations)=measured(||om9_cage_create_placed(&d,IDENTITY.as_ptr(),&mut out));
        assert_eq!(status,-1);assert!(out.is_null());assert_eq!(allocations,0,"bad basis{which} was copied before admission");
    }
}}
#[test]
fn descriptor_query_checks_header_basis_frame_region_and_retains_no_data(){unsafe{
    let c=cage();let r=descriptor(&c);assert_eq!(measured(||om9_cage_validate_descriptor(&r,IDENTITY.as_ptr())),(0,0));
    let mut d=descriptor(&c);d.counts=[usize::MAX;3];d.knots=[std::ptr::null();3];d.points=std::ptr::null();d.weights=std::ptr::null();
    assert_eq!(measured(||om9_cage_validate_descriptor(&d,IDENTITY.as_ptr())),(-1,0));
    d=descriptor(&c);d.point_count=9;assert_eq!(measured(||om9_cage_validate_descriptor(&d,IDENTITY.as_ptr())),(-6,0));
    d=descriptor(&c);d.region=2;assert_eq!(measured(||om9_cage_validate_descriptor(&d,IDENTITY.as_ptr())),(-3,0));
    d=descriptor(&c);d.region=1;d.falloff=-1.;assert_eq!(measured(||om9_cage_validate_descriptor(&d,IDENTITY.as_ptr())),(-3,0));
    let mut frame=IDENTITY;frame[0]=0.;assert_eq!(measured(||om9_cage_validate_descriptor(&r,frame.as_ptr())),(-2,0));
    assert_eq!(om9_cage_validate_descriptor(std::ptr::null(),IDENTITY.as_ptr()),-6);
}}
#[test]
fn binding_budget_values_and_signed_triangle_indices_are_admitted_without_copy(){unsafe{
    let source=[[0.;3],[1.,0.,0.],[0.,1.,0.]];let params=source;let faces=[0i32,1,2];
    let admit=|n,params_count,indices:&[i32],mesh|om9_cage_binding_admit(source.as_ptr(),n,params.as_ptr(),params_count,indices.as_ptr().cast(),indices.len(),4,mesh);
    assert_eq!(measured(||admit(3,3,&faces,true)),(0,0));
    assert_eq!(measured(||admit(3,2,&faces,true)),(-6,0));
    for indices in [[-1,1,2],[0,0,2],[0,1,3]]{assert_eq!(measured(||admit(3,3,&indices,true)),(-13,0));}
    assert_eq!(measured(||admit(3,3,&[0,1],true)),(-13,0));
    assert_eq!(measured(||admit(0,0,&[],true)),(-13,0));
    assert_eq!(measured(||admit(usize::MAX,usize::MAX,&[],false)),(-7,0));
    assert_eq!(measured(||om9_cage_binding_admit(std::ptr::null(),0,std::ptr::null(),0,std::ptr::null(),0,4,false)),(0,0));
    let bad=[[f64::NAN;3];3];assert_eq!(measured(||om9_cage_binding_admit(bad.as_ptr(),3,params.as_ptr(),3,faces.as_ptr().cast(),3,4,true)),(-4,0));
    let faces64=[0i64,1,2];assert_eq!(measured(||om9_cage_binding_admit(source.as_ptr(),3,params.as_ptr(),3,faces64.as_ptr().cast(),3,8,true)),(0,0));
}}
