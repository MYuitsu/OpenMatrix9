// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature="3dm-brep")]
use std::{alloc::{GlobalAlloc,Layout,System},sync::atomic::{AtomicBool,Ordering}};

struct FailOneAllocation;
static FAIL_NEXT:AtomicBool=AtomicBool::new(false);
unsafe impl GlobalAlloc for FailOneAllocation {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        if FAIL_NEXT.swap(false,Ordering::SeqCst) {std::ptr::null_mut()} else {unsafe {System.alloc(layout)}}
    }
    unsafe fn dealloc(&self,ptr:*mut u8,layout:Layout) {unsafe {System.dealloc(ptr,layout)}}
    unsafe fn realloc(&self,ptr:*mut u8,layout:Layout,size:usize)->*mut u8 {
        if FAIL_NEXT.swap(false,Ordering::SeqCst) {std::ptr::null_mut()} else {unsafe {System.realloc(ptr,layout,size)}}
    }
}
#[global_allocator]
static ALLOCATOR:FailOneAllocation=FailOneAllocation;

#[test]
fn native_source_copy_reports_real_rust_allocation_failure_as_memory() {
    let limits=om9_core::budget::Limits::default();
    let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,0.001,&limits).unwrap();
    let mut armed=false;
    let result=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,0.001,&limits,|_,pending| {
        if pending>0&&!armed {armed=true;FAIL_NEXT.store(true,Ordering::SeqCst);}Ok(())
    });
    FAIL_NEXT.store(false,Ordering::SeqCst);
    assert!(armed);assert!(matches!(result,Err(om9_core::Error::Budget("bytes"))));
    let source=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,0.001,&limits,|_,_|Ok(())).unwrap();
    assert!(!source.is_empty());
    let mut edge_armed=false;
    let edges=om9_kernel::editing::all_edges_from_source_admitted(&source,&limits,|_,pending| {
        if pending>0&&!edge_armed {edge_armed=true;FAIL_NEXT.store(true,Ordering::SeqCst);}Ok(())
    });
    FAIL_NEXT.store(false,Ordering::SeqCst);
    assert!(edge_armed);assert!(matches!(edges,Err(om9_kernel::editing::EditingError::Budget(message)) if message=="edge curve allocation"));
    let tight=om9_core::budget::Limits {max_outputs:1,..limits};
    let limited=om9_kernel::editing::all_edges_from_source_admitted(&source,&tight,|_,_|Ok(()));
    assert!(matches!(limited,Err(om9_kernel::editing::EditingError::Budget(message)) if message!="edge curve allocation"));
    let refused=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,0.001,&limits,|_,_|Err(om9_core::Error::Budget("managed memory")));
    assert!(matches!(refused,Err(om9_core::Error::Budget("managed memory"))));
    let corrupt=om9_kernel::brep::from_3dm_source_typed_admitted(&[1,2,3,4],0.001,&limits,|_,_|Ok(()));
    assert!(matches!(corrupt,Err(om9_core::Error::Io(_))));
}
