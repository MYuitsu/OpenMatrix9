use openmatrix9_rust::topology_snapshot::*;
use openmatrix9_rust::topology_snapshot_ffi::*;

#[test]
fn abi_layout_matches_the_native_x64_companion() {
    use std::mem::{size_of,align_of,offset_of};
    assert_eq!((size_of::<Face>(),align_of::<Face>(),offset_of!(Face,first_loop)),(32,8,16));
    assert_eq!((size_of::<Loop>(),align_of::<Loop>(),offset_of!(Loop,first_use)),(24,8,8));
    assert_eq!((size_of::<Use>(),align_of::<Use>(),offset_of!(Use,domain)),(96,8,48));
    assert_eq!((size_of::<Summary>(),align_of::<Summary>(),offset_of!(Summary,vertex_manifold)),(52,4,36));
}
#[test]
fn surface_flags_are_checked_before_atomic_native_output(){
 let f=Face{surface_id:1,orientation:1,reserved:0,first_loop:0,loop_count:1};
 let l=Loop{role:1,reserved:0,first_use:0,use_count:1};
 let u=Use{edge_id:1,pcurve_id:1,start_vertex:1,end_vertex:1,orientation:1,seam:0,degenerate:0,reserved:0,domain:[0.,1.],uv_start:[0.,0.],uv_end:[0.,0.]};
 let mut output=[77];let call=|u:&Use,out:*mut u32|unsafe{om9_topology_inspect(&f,1,&l,1,u,1,out,1)};
 let r=call(&u,output.as_mut_ptr());assert_eq!((r.status,r.vertex_manifold,r.face_oriented,r.vertex_count,r.boundary_vertex_count),(0,1,1,1,1));assert_eq!(output,[1]);
 let bad=Use{domain:[1.,0.],..u};output=[77];assert_ne!(call(&bad,output.as_mut_ptr()).status,0);assert_eq!(output,[77]);
}

#[test]
fn bad_pointers_counts_and_insufficient_output_reject_without_writes() {
    let mut output=[0xfeed_u32;4];
    let face=Face{surface_id:1,orientation:1,reserved:0,first_loop:0,loop_count:1};
    let l=Loop{role:1,reserved:0,first_use:0,use_count:1};
    let u=Use{edge_id:1,pcurve_id:1,start_vertex:1,end_vertex:1,orientation:1,seam:0,degenerate:0,reserved:0,
              domain:[0.,1.],uv_start:[0.,0.],uv_end:[0.,0.]};
    unsafe {
        let r=om9_topology_inspect(std::ptr::null(),1,&l,1,&u,1,output.as_mut_ptr(),4);
        assert_ne!(r.status,0);
        let r=om9_topology_inspect(&face,usize::MAX,&l,1,&u,1,output.as_mut_ptr(),4);
        assert_ne!(r.status,0);
        let r=om9_topology_inspect(&face,1,&l,1,&u,1,output.as_mut_ptr(),0);
        assert_ne!(r.status,0);
        let bad=(std::ptr::addr_of!(u) as *const u8).add(1) as *const Use;
        assert_ne!(om9_topology_inspect(&face,1,&l,1,bad,1,output.as_mut_ptr(),4).status,0);
    }
    assert_eq!(output,[0xfeed;4]);
}

#[test]
fn one_free_circle_returns_component_and_invalid_uv_is_atomic() {
    let f=Face{surface_id:1,orientation:1,reserved:0,first_loop:0,loop_count:1};
    let l=Loop{role:1,reserved:0,first_use:0,use_count:1};
    let mut u=Use{edge_id:1,pcurve_id:1,start_vertex:1,end_vertex:1,orientation:1,seam:0,degenerate:0,reserved:0,
                 domain:[0.,1.],uv_start:[0.,0.],uv_end:[0.,0.]};
    let mut output=[99];
    let r=unsafe{om9_topology_inspect(&f,1,&l,1,&u,1,output.as_mut_ptr(),1)};
    assert_eq!((r.status,r.edge_closed,r.edge_manifold,r.boundary_components),(0,0,1,1));
    assert_eq!(output,[1]);output=[99];u.uv_end[0]=f64::INFINITY;
    assert_ne!(unsafe{om9_topology_inspect(&f,1,&l,1,&u,1,output.as_mut_ptr(),1)}.status,0);
    assert_eq!(output,[99]);
}

#[test]
fn native_restriction_domains_are_rejected_before_curve_evaluation() {
    assert_eq!(om9_topology_domain(0.,1.),0);
    for (a,b) in [(f64::NAN,1.),(0.,f64::INFINITY),(1.,0.),(-f64::MAX,f64::MAX)] {
        assert_ne!(om9_topology_domain(a,b),0);
    }
}
