use openmatrix9_rust::{surface_complex::capture,topology_snapshot::{Face,Loop,Snapshot,Use}};
fn triangles(rows:&[[u64;3]])->(Vec<Face>,Vec<Loop>,Vec<Use>){
 let mut f=Vec::new();let mut l=Vec::new();let mut u=Vec::new();
 for (i,t) in rows.iter().enumerate(){
  f.push(Face{surface_id:i as u64+1,orientation:1,reserved:0,first_loop:i,loop_count:1});
  l.push(Loop{role:1,reserved:0,first_use:u.len(),use_count:3});
  for (a,b) in [(t[0],t[1]),(t[1],t[2]),(t[2],t[0])]{u.push(Use{edge_id:a.min(b)*16+a.max(b),pcurve_id:u.len() as u64+1,start_vertex:a,end_vertex:b,orientation:if a<b{1}else{2},seam:0,degenerate:0,reserved:0,domain:[0.,1.],uv_start:[0.,0.],uv_end:[1.,0.]});}
 }
 (f,l,u)
}
fn periodic(poles:bool,torus:bool)->(Vec<Face>,Vec<Loop>,Vec<Use>){
 let edge=|id,p,a,b,side,seam,degenerate|Use{edge_id:id,pcurve_id:p,start_vertex:a,end_vertex:b,orientation:side,seam,degenerate,reserved:0,domain:[0.,1.],uv_start:[0.,0.],uv_end:[1.,0.]};
 let uses=if torus {vec![edge(1,1,1,1,1,1,0),edge(2,2,1,1,1,1,0),edge(1,3,1,1,2,1,0),edge(2,4,1,1,2,1,0)]}
 else{vec![edge(1,1,1,1,1,0,u32::from(poles)),edge(2,2,1,2,1,1,0),edge(3,3,2,2,1,0,u32::from(poles)),edge(2,4,2,1,2,1,0)]};
 (vec![Face{surface_id:1,orientation:1,reserved:0,first_loop:0,loop_count:1}],vec![Loop{role:1,reserved:0,first_use:0,use_count:4}],uses)
}
#[test]
fn disk_has_interval_vertex_links_and_closed_tetrahedron_has_cycles(){
 let (f,l,u)=triangles(&[[1,2,3]]);let s=capture(&f,&l,&u).unwrap();
 assert!(s.vertex_manifold&&s.face_oriented);assert_eq!((s.vertex_count,s.boundary_vertex_count),(3,3));
 let (f,l,u)=triangles(&[[1,3,2],[1,2,4],[2,3,4],[3,1,4]]);let s=capture(&f,&l,&u).unwrap();
 assert!(s.vertex_manifold&&s.face_oriented);assert_eq!((s.vertex_count,s.boundary_vertex_count),(4,0));
}
#[test]
fn closed_edge_incidence_does_not_accept_a_vertex_pinch(){
 let (f,l,u)=triangles(&[[1,3,2],[1,2,4],[2,3,4],[3,1,4],[1,6,5],[1,5,7],[5,6,7],[6,1,7]]);
 let s=Snapshot::capture(&f,&l,&u).unwrap();assert!(s.edge_manifold&&s.edge_closed);
 let s=capture(&f,&l,&u).unwrap();assert!(!s.vertex_manifold);assert!(s.face_oriented);assert_eq!(s.vertex_count,7);
}
#[test]
fn paired_seams_preserve_distinct_endpoint_germs_and_degenerate_poles(){
 for (poles,torus,vertices,boundary) in [(false,false,2,2),(true,false,2,0),(false,true,1,0)]{
  let (f,l,u)=periodic(poles,torus);let s=capture(&f,&l,&u).unwrap();
  assert!(s.vertex_manifold&&s.face_oriented,"poles={poles}, torus={torus}");assert_eq!((s.vertex_count,s.boundary_vertex_count),(vertices,boundary));
 }
}
#[test]
fn branching_edges_and_mixed_face_orientation_are_separate_failures(){
 let tetra=[[1,3,2],[1,2,4],[2,3,4],[3,1,4]];
 let (mut f,l,u)=triangles(&tetra);f[0].orientation=2;let s=capture(&f,&l,&u).unwrap();assert!(s.vertex_manifold);assert!(!s.face_oriented);
 for face in &mut f{face.orientation=2;}let s=capture(&f,&l,&u).unwrap();assert!(s.vertex_manifold&&s.face_oriented);
 let (f,l,u)=triangles(&[[1,3,2],[1,2,4],[2,3,4],[3,1,4],[1,2,5]]);let s=capture(&f,&l,&u).unwrap();assert!(!s.vertex_manifold&&!s.face_oriented);
}
#[test]
fn collapsed_loops_and_malformed_rows_cannot_supply_a_manifold_claim(){
 let (f,l,mut u)=triangles(&[[1,1,1]]);for row in &mut u{row.degenerate=1;}
 let s=capture(&f,&l,&u).unwrap();assert!(!s.vertex_manifold);assert_eq!(s.vertex_count,1);
 let (mut f,l,u)=triangles(&[[1,2,3]]);f[0].first_loop=usize::MAX;assert!(capture(&f,&l,&u).is_err());
 let (f,l,mut u)=triangles(&[[1,2,3]]);u[0].domain[0]=f64::NAN;assert!(capture(&f,&l,&u).is_err());
 assert!(capture(&vec![f[0];16385],&[],&[]).is_err());
}
#[test]
fn collapsed_face_at_an_existing_valid_vertex_is_not_silently_discarded(){
 for rows in [&[[1,2,3]][..],&[[1,3,2],[1,2,4],[2,3,4],[3,1,4]][..]]{
  let (mut f,mut l,mut u)=triangles(rows);
  f.push(Face{surface_id:f.len() as u64+1,orientation:1,reserved:0,first_loop:l.len(),loop_count:1});
  l.push(Loop{role:1,reserved:0,first_use:u.len(),use_count:1});
  u.push(Use{edge_id:99,pcurve_id:u.len() as u64+1,start_vertex:1,end_vertex:1,orientation:1,seam:0,degenerate:1,reserved:0,domain:[0.,1.],uv_start:[0.,0.],uv_end:[0.,0.]});
  let s=capture(&f,&l,&u).unwrap();assert!(!s.vertex_manifold);
  assert_eq!(s.boundary_vertex_count,if rows.len()==1{2}else{0});
  if rows.len()>1{let raw=Snapshot::capture(&f,&l,&u).unwrap();assert!(raw.edge_manifold&&raw.edge_closed);}
 }
}
