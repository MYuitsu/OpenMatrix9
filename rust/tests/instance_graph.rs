use openmatrix9_rust::core_objects::Stamp;
use openmatrix9_rust::geometry_frame::IDENTITY;
use openmatrix9_rust::instance_graph::{Graph,Node,ROOT};
use openmatrix9_rust::instance_graph_ffi::{om9_instance_graph_json,om9_instance_graph_counts};
fn node(parent:usize,representation:u32)->Node {
    Node{stamp:Stamp{document:1,object:2,generation:3,revision:4,epoch:5},parent,representation,reserved:0,local_to_world:IDENTITY}
}
#[test]
fn graph_owns_ordered_occurrences_including_shared_definition_identities() {
    let mut nodes=[node(ROOT,11),node(0,10),node(1,2),node(0,10),node(3,2)];nodes[4].local_to_world[3]=7.;nodes[4].local_to_world[0]=-2.;
    let graph=Graph::capture(&nodes).unwrap();nodes[4].local_to_world[3]=99.;assert_eq!(nodes[4].local_to_world[3],99.);
    let json=graph.json().unwrap();assert!(json.contains("\"representation\":\"group\""));assert!(json.contains("\"world_orientation_sign\":-1"));assert!(json.contains("\"object\":\"2\""));assert!(!json.contains("99"));
    assert!(Graph::capture(&[node(ROOT,11)]).is_ok());
}
#[test]
fn invalid_structure_identity_depth_and_frames_are_rejected_before_owning() {
    assert!(Graph::capture(&[]).is_err());
    for mut nodes in [vec![node(0,11)],vec![node(ROOT,2)],vec![node(ROOT,11),node(1,2)],vec![node(ROOT,11),node(0,2),node(1,2)],vec![node(ROOT,11),node(ROOT,2)]] {
        assert!(Graph::capture(&nodes).is_err());nodes.clear();
    }
    let mut n=node(ROOT,11);n.stamp.epoch=0;assert!(Graph::capture(&[n]).is_err());
    n=node(ROOT,11);n.reserved=1;assert!(Graph::capture(&[n]).is_err());
    n=node(ROOT,11);n.local_to_world[0]=0.;assert!(Graph::capture(&[n]).is_err());
    let mut deep=vec![node(ROOT,11)];for i in 1..65{deep.push(node(i-1,11));}assert!(Graph::capture(&deep).is_err());
    assert!(Graph::capture(&vec![node(ROOT,11);4097]).is_err());
}
#[test]
fn bounded_graph_abi_serialization_and_invalid_envelopes_are_atomic() {
    assert_eq!(std::mem::size_of::<Node>(),184);assert_eq!(std::mem::align_of::<Node>(),8);
    assert_eq!(om9_instance_graph_counts(4096,64),0);assert_ne!(om9_instance_graph_counts(4097,1),0);assert_ne!(om9_instance_graph_counts(1,65),0);
    let nodes=[node(ROOT,11),node(0,2)];
    let call=|p,n,out,cap|unsafe{om9_instance_graph_json(p,n,out,cap)};
    let query=call(nodes.as_ptr(),nodes.len(),std::ptr::null_mut(),0);assert_eq!(query.status,0);
    let mut bytes=vec![0x5a;query.needed+1];assert_eq!(call(nodes.as_ptr(),nodes.len(),bytes.as_mut_ptr(),query.needed).status,0);
    let before=bytes.clone();assert_eq!(call(nodes.as_ptr(),nodes.len(),bytes.as_mut_ptr(),query.needed-1).status,1001);assert_eq!(bytes,before);
    for (p,n) in [(std::ptr::null(),1),(1usize as _,4097),((nodes.as_ptr() as *const u8).wrapping_add(1) as _,2),(nodes.as_ptr(),usize::MAX)] {
        assert_ne!(call(p,n,bytes.as_mut_ptr(),query.needed).status,0);assert_eq!(bytes,before);
    }
}
