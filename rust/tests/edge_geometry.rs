use openmatrix9_rust::edge_geometry::{Edge,Vertex,Geometry,Error,counts_allowed,tolerance_valid};
use openmatrix9_rust::edge_geometry_ffi::{Input,om9_edge_geometry_json};
use openmatrix9_rust::geometry_frame::IDENTITY;
fn vertices()->[Vertex;2]{[Vertex{vertex_id:1,point:[1.,2.,3.],tolerance:1e-7,location:IDENTITY},Vertex{vertex_id:2,point:[4.,5.,6.],tolerance:0.,location:IDENTITY}]}
fn edge()->Edge{Edge{edge_id:1,start_vertex:1,end_vertex:2,domain:[2.,5.],tolerance:1e-7,curve_present:1,degenerate:0,same_parameter:1,same_range:0,location:IDENTITY}}
#[test]
fn owned_edges_vertices_and_absent_degenerate_domains_stay_distinct(){
 let mut vs=vertices();let mut es=[edge()];let owned=Geometry::capture(&es,&vs).unwrap();vs[0].point=[99.;3];es[0].domain=[10.,11.];
 assert_eq!(vs[0].point,[99.;3]);assert_eq!(es[0].domain,[10.,11.]);
 assert_eq!(owned.vertices()[0].point,[1.,2.,3.]);assert_eq!(owned.edges()[0].domain,[2.,5.]);
 assert!(owned.json().unwrap().contains("\"native_same_range\":false"));
 assert!(owned.json().unwrap().contains("\"parameter_correspondence_certificate\":null"));
 let absent=Geometry::capture(&[Edge{end_vertex:1,curve_present:0,degenerate:1,domain:[0.,0.],..edge()}],&vertices()).unwrap();
 assert!(absent.json().unwrap().contains("\"domain\":null"));
 let empty=Geometry::capture(&[],&[]).unwrap().json().unwrap();assert!(empty.contains("\"edges_3d\":[]"));assert!(empty.contains("\"vertices\":[]"));
}
#[test]
fn malformed_edge_vertex_identity_values_flags_and_frames_reject(){
 let vs=vertices();
 for bad in [Edge{edge_id:0,..edge()},Edge{end_vertex:3,..edge()}]{assert_eq!(Geometry::capture(&[bad],&vs).unwrap_err(),Error::Identity);}
 assert_eq!(Geometry::capture(&[edge(),edge()],&vs).unwrap_err(),Error::Identity);
 for flag in [Edge{same_range:2,..edge()},Edge{same_parameter:2,..edge()},Edge{curve_present:2,..edge()},Edge{degenerate:2,..edge()}]{assert_eq!(Geometry::capture(&[flag],&vs).unwrap_err(),Error::Flags);}
 for tolerance in [-1.,f64::NAN,f64::INFINITY,1e151]{assert!(!tolerance_valid(tolerance));assert_eq!(Geometry::capture(&[Edge{tolerance,..edge()}],&vs).unwrap_err(),Error::Tolerance);}
 assert!(tolerance_valid(0.));
 for domain in [[5.,2.],[2.,2.],[f64::NAN,2.]]{assert_eq!(Geometry::capture(&[Edge{domain,..edge()}],&vs).unwrap_err(),Error::Domain);}
 assert_eq!(Geometry::capture(&[Edge{curve_present:0,domain:[0.,0.],..edge()}],&vs).unwrap_err(),Error::MissingCurve);
 let mut bad=vs;bad[1].point[0]=f64::NAN;assert_eq!(Geometry::capture(&[edge()],&bad).unwrap_err(),Error::Coordinate);
 let mut singular=IDENTITY;singular[0]=0.;assert_eq!(Geometry::capture(&[Edge{location:singular,..edge()}],&vs).unwrap_err(),Error::Location);
 let mut bad=vs;bad[0].vertex_id=0;assert_eq!(Geometry::capture(&[edge()],&bad).unwrap_err(),Error::Identity);
 let mut bad=vs;bad[0].tolerance=-1.;assert_eq!(Geometry::capture(&[edge()],&bad).unwrap_err(),Error::Tolerance);
 let mut bad=vs;bad[0].location=singular;assert_eq!(Geometry::capture(&[edge()],&bad).unwrap_err(),Error::Location);
 assert!(!counts_allowed(usize::MAX,1));assert!(!counts_allowed(16384,1));
}
#[test]
fn edge_geometry_abi_copies_atomic_json_and_guards_live_buffers(){
 assert_eq!(std::mem::size_of::<Edge>(),192);assert_eq!(std::mem::size_of::<Vertex>(),168);assert_eq!(std::mem::size_of::<Input>(),32);
 let es=[edge()];let vs=vertices();let input=Input{edges:es.as_ptr(),edge_count:1,vertices:vs.as_ptr(),vertex_count:2};
 let call=|i,out,cap|unsafe{om9_edge_geometry_json(i,out,cap)};
 let q=call(input,std::ptr::null_mut(),0);assert_eq!(q.status,0);
 let mut out=vec![0x5a;q.needed+1];assert_eq!(call(input,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out[q.needed],0x5a);let saved=out.clone();
 assert_eq!(call(input,out.as_mut_ptr(),q.needed-1).status,1001);assert_eq!(out,saved);
 for bad in [Input{edge_count:usize::MAX,..input},Input{edges:std::ptr::null(),..input},Input{vertices:(vs.as_ptr() as *const u8).wrapping_add(1) as _,..input}]{assert_ne!(call(bad,out.as_mut_ptr(),q.needed).status,0);assert_eq!(out,saved);}
 let invalid=[Edge{tolerance:f64::NAN,..edge()}];assert_eq!(call(Input{edges:invalid.as_ptr(),..input},out.as_mut_ptr(),q.needed).status,8001);assert_eq!(out,saved);
 assert_eq!(call(input,std::ptr::null_mut(),1).status,1002);assert_eq!(call(input,out.as_mut_ptr(),usize::MAX).status,1002);assert_eq!(out,saved);
 assert_eq!(call(Input{edges:std::ptr::null(),edge_count:0,vertices:std::ptr::null(),vertex_count:0},std::ptr::null_mut(),0).status,0);
}
#[test]
fn count_valid_but_oversized_json_leaves_ffi_output_untouched(){
 let mut location=IDENTITY;location[3]=1e149;location[7]=1e149;location[11]=1e149;
 let es:Vec<_>=(1..=16000).map(|id|Edge{edge_id:id,tolerance:1e149,location,..edge()}).collect();let vs=vertices();assert!(counts_allowed(es.len(),vs.len()));
 let mut out=[0x5a;64];let before=out;
 let q=unsafe{om9_edge_geometry_json(Input{edges:es.as_ptr(),edge_count:es.len(),vertices:vs.as_ptr(),vertex_count:vs.len()},out.as_mut_ptr(),out.len())};
 assert_eq!(q.status,1000);assert_eq!(q.needed,0);assert_eq!(out,before);
}
