// SPDX-License-Identifier: LGPL-2.1-or-later
//! Schema/branch tests; the separate native fixture proves actual master geometry.
use super::CurveSession;
use crate::builder_history::{Json,parse_document};
use crate::prepared_result::{map,string,encode_owned};
#[path="curve_native_selection_tests.rs"]
mod selection_tests;
#[path="curve_native_parameter_tests.rs"]
mod parameter_tests;
#[path="curve_native_contact_tests.rs"]
mod contact_tests;
fn get<'a>(v:&'a Json,k:&str)->&'a Json{let Json::Object(o)=v else{panic!("object")};o.get(k).unwrap_or_else(||panic!("missing {k}"))}
fn rows(v:&Json)->&[Json]{let Json::Array(a)=v else{panic!("rows")};a}
const UNITS:&str="1|1|1e0|1e0|1e-2|1e-3|1e-2";
fn point(p:[f64;3])->Json{super::input_log::vector(&p)}
pub(super) fn source(name:&str,role:&str,begin:usize,end:usize)->(Json,Json){
 let attrs=format!(r#"{{"schema_version":1,"document_uid":"uid","object_name":"{name}","native_type":"Part::Feature","label":"Owned source","label2":null,"view_provider_type":null,"visibility":true,"selectable":true,"shape_appearance":null,"coverage":{{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}}}"#);
 let raw=format!(r#"{{"document":"1","object":"2","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"uid","object_name":"{name}","subelement":"Edge1","unit_context_snapshot":"{UNITS}","world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[],"role":"{role}","role_index":0,"point_begin":{begin},"point_end":{end},"snapshot_frame":"document_world","input_snapshot_index":1}}"#);
 let snap=format!(r#"{{"schema_version":1,"native_shape_type":6,"ownership":"private_native_copy","unit_context_snapshot":"{UNITS}","geometry_data":{{"representation":"curve","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 unit schema fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":false,"closed":false}}}},"source_attributes":{attrs},"selected_points_world":[[1,0,0],[0,1,0],[-1,0,0]]}}"#);
 let mut snapshot=parse_document(&snap,100_000,10_000).unwrap();
 let Json::Object(o)=&mut snapshot else{panic!()};let Json::Object(data)=o.get_mut("geometry_data").unwrap()else{panic!()};
 data.insert("native_root_location".into(),Json::Array(vec![point4([1.,0.,0.,0.]),point4([0.,1.,0.,0.]),point4([0.,0.,1.,0.]),point4([0.,0.,0.,1.])]));
 data.insert("native_root_orientation".into(),string("forward"));
 let mut reference=parse_document(&raw,100_000,10_000).unwrap();
 if role=="fit_points"{
  // Complete literal CAD vertex compound. Native FitPoints never selects
  // Edge1 to obtain an arbitrary three-point batch.
  let Json::Object(r)=&mut reference else{panic!()};r.insert("subelement".into(),string(""));
  let Json::Object(s)=&mut snapshot else{panic!()};s.insert("native_shape_type".into(),Json::Number(0.));
  let Json::Object(g)=s.get_mut("geometry_data").unwrap()else{panic!()};g.insert("representation".into(),string("brep"));
  let Json::Object(t)=s.get_mut("topology_data").unwrap()else{panic!()};
  t.insert("vertices".into(),Json::Array([[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]].into_iter().enumerate().map(|(i,p)|literal_vertex(i+1,p)).collect()));
 }
 if role=="path"{
  let Json::Object(s)=&mut snapshot else{panic!()};let Json::Object(t)=s.get_mut("topology_data").unwrap()else{panic!()};
  let edge=r#"[{"native_topology_index":"1","curve_present":true,"degenerate":false,"geometry_coordinates":"native_local","native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"domain":[0,20],"geometry_data":{"schema_version":1,"kind":"analytic_curve","analytic_type":"line","dimension":3,"parameter_convention":"occt_native","origin_xyz":[10,20,30],"axes_xyz":[[0,0,1],[0,0,0],[0,0,0]],"parameters":{},"natural_domains":["unbounded"]}}]"#;
  t.insert("edges_3d".into(),parse_document(edge,10000,1000).unwrap());
 }
 (reference,snapshot)
}
fn point4(p:[f64;4])->Json{super::input_log::vector(&p)}
fn literal_vertex(index:usize,p:[f64;3])->Json{
 map([("vertex_id",string(&index.to_string())),("native_topology_index",string(&index.to_string())),("point_xyz",point(p)),("tolerance",Json::Number(1e-7)),
 ("geometry_coordinates",string("native_local")),("native_location",Json::Array(vec![point4([1.,0.,0.,0.]),point4([0.,1.,0.,0.]),point4([0.,0.,1.,0.]),point4([0.,0.,0.,1.])]))])
}
fn point_source(name:&str,begin:usize,p:[f64;3])->(Json,Json){
 let(mut r,mut s)=source(name,"point",begin,begin+1);let Json::Object(r)=&mut r else{panic!()};r.insert("subelement".into(),string("Vertex1"));
 let Json::Object(o)=&mut s else{panic!()};o.insert("selected_points_world".into(),Json::Array(vec![point(p)]));o.insert("native_shape_type".into(),Json::Number(7.));
 let Json::Object(g)=o.get_mut("geometry_data").unwrap()else{panic!()};g.insert("representation".into(),string("point"));
 let Json::Object(t)=o.get_mut("topology_data").unwrap()else{panic!()};t.insert("vertices".into(),Json::Array(vec![literal_vertex(1,p)]));
 (Json::Object(r.clone()),s)
}
pub(super) fn request(s:&CurveSession,command:&str,action:&str,text:&str,kind:u32,p:[f64;3],sources:Option<(Json,Json)>)->String{
 let (refs,snaps)=sources.map_or((vec![],vec![]),|(r,v)|(vec![r],vec![v]));
 encode_owned(&map([("schema_version",Json::Number(1.)),("operation",string("AcceptCurveNativeInput")),
 ("command",string(command)),("generation",string(&u64::MAX.to_string())),("context_before",string(&s.circle_input_context_key().to_string())),
 ("admission_revision",string(&s.native_input_revision().to_string())),
 ("unit_context_snapshot",string(UNITS)),("action",string(action)),("text",string(text)),
 ("reference_kind",Json::Number(kind as f64)),("point_world",point(p)),("normal",point([0.,0.,1.])),
 ("points_world",if action=="fit_batch"{super::input_log::points(&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]])}else{Json::Array(vec![])}),
 ("source_refs",Json::Array(refs)),("source_snapshots",Json::Array(snaps))])).unwrap()
}
fn evidence(s:&CurveSession)->Json{parse_document(&s.construction_evidence(u64::MAX).unwrap(),16*1024*1024,1_000_000).unwrap()}
#[test]
fn original_native_tokens_and_owned_parent_source_bind_the_same_accepted_event(){
 for name in ["Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(name).unwrap();s.input("AroundCurve").unwrap();
  let raw=request(&s,name,"reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0)));
  s.accept_native_json(&raw,u64::MAX).unwrap();
  let raw=request(&s,name,"reference","OnCurve=0.2500",2,[10.,20.,35.],None);
  s.accept_native_json(&raw,u64::MAX).unwrap();s.input("5mm").unwrap();if name=="Ellipse"{s.input("3mm").unwrap();}
  let v=evidence(&s);let events=rows(get(&v,"accepted_input_events"));
  assert_eq!(get(&events[1],"text"),&string("Curve=Source.Edge1"));assert_eq!(get(&events[2],"text"),&string("OnCurve=0.2500"));
  let refs=rows(get(&v,"source_refs"));let snaps=rows(get(&v,"source_snapshots"));assert_eq!(refs.len(),1);assert_eq!(snaps.len(),1);
  assert_eq!(get(&refs[0],"revision"),&string(&u64::MAX.to_string()));assert_eq!(get(&refs[0],"accepted_event_index"),&Json::Number(1.));
  assert_eq!(get(get(&snaps[0],"source_attributes"),"label"),&string("Owned source"));
 }
}
#[test]
fn advertised_source_indexing_resolves_owned_native_sources_without_a_numeric_slot(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 for (name,begin) in [("First",0),("Second",3)]{
  let raw=request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(source(name,"fit_points",begin,begin+3)));
  s.accept_native_json(&raw,u64::MAX).unwrap();
 }
 s.input("").unwrap();let v=evidence(&s);let refs=rows(get(&v,"source_refs"));let snaps=rows(get(&v,"source_snapshots"));
 assert_eq!(snaps.len(),2);
 // An independent consumer follows the advertised collection/base. There is
 // no numeric snapshot in this constructor value; both native owners resolve.
 for (r,want) in refs.iter().zip(["First","Second"]){
  let Json::Number(index)=get(r,"input_snapshot_index") else{panic!("index")};
  let snapshot=match get(&v,"source_snapshot_indexing"){
   Json::String(mode) if mode=="source_snapshots_one_based" => snaps.get(*index as usize-1),
   Json::String(mode) if mode=="input_snapshots_zero_based_including_numeric_input" =>
    if let Json::Object(o)=&v{o.get("input_snapshots").and_then(|a|rows(a).get(*index as usize))}else{None},
   _=>None,
  }.expect("advertised source indexing must resolve its native owner");
  assert_eq!(get(get(snapshot,"source_attributes"),"object_name"),&string(want));
 }
}
#[test]
fn fit_undo_releases_the_entire_native_source_branch_and_preserves_replacement(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 let raw=request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(source("RemovedCloud","fit_points",0,3)));
 s.accept_native_json(&raw,u64::MAX).unwrap();s.input("Undo").unwrap();
 let raw=request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(source("AcceptedCloud","fit_points",0,3)));
 s.accept_native_json(&raw,u64::MAX).unwrap();s.input("").unwrap();let v=evidence(&s);
 assert_eq!(rows(get(&v,"source_refs")).len(),1);assert_eq!(get(&rows(get(&v,"source_refs"))[0],"object_name"),&string("AcceptedCloud"));
 assert!(!s.construction_evidence(u64::MAX).unwrap().contains("RemovedCloud"));
}
#[test]
fn repeated_native_batches_share_one_owned_snapshot_with_distinct_ordered_ranges(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 for begin in [0,3]{let raw=request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(source("Source","fit_points",begin,begin+3)));s.accept_native_json(&raw,u64::MAX).unwrap();}
 s.input("").unwrap();let v=evidence(&s);let refs=rows(get(&v,"source_refs"));assert_eq!(refs.len(),2);assert_eq!(rows(get(&v,"source_snapshots")).len(),1);
 assert_eq!(get(&refs[0],"input_snapshot_index"),&Json::Number(1.));assert_eq!(get(&refs[1],"input_snapshot_index"),&Json::Number(1.));
 assert_eq!(get(&refs[1],"point_begin"),&Json::Number(3.));
}
#[test]
fn stale_or_malformed_native_envelopes_reject_before_points_options_context_or_log_change(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("AroundCurve").unwrap();
 let raw=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0)));
 let before=s.prompt();let log=s.accepted_input_log.json();let key=s.circle_input_context_key();
 for bad in [raw.replace(&u64::MAX.to_string(),"18446744073709551614"),raw.replacen("\"context_before\":\"1\"","\"context_before\":\"0\"",1),raw.replacen("\"object_name\":\"Source\"","\"object_name\":\"Other\"",1),raw.replacen("\"input_snapshot_index\":1","\"input_snapshot_index\":2",1),raw.replacen("\"ownership\":\"private_native_copy\"","\"ownership\":\"borrowed\"",1),raw.replacen("\"role\":\"path\"","\"role\":\"fit_points\"",1)]{
  assert!(s.accept_native_json(&bad,u64::MAX).is_err());assert_eq!(s.prompt(),before);assert_eq!(s.accepted_input_log.json(),log);assert_eq!(s.circle_input_context_key(),key);assert!(s.points().is_empty());
 }
 s.accept_native_json(&raw,u64::MAX).unwrap();assert_eq!(s.circle_input_context_key(),key+1);
}
#[test]
fn native_selection_cannot_admit_an_unowned_source_even_when_solver_ignores_fields(){
 for name in ["Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(name).unwrap();s.input("AroundCurve").unwrap();let before=s.prompt();
  let raw=request(&s,name,"reference","Curve=Unowned.Edge1",1,[0.;3],None);
  assert!(s.accept_native_json(&raw,u64::MAX).is_err());assert_eq!(s.prompt(),before);
 }
}
#[test]
fn duplicate_snapshots_within_one_batch_are_interned_once(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 let(a,snapshot)=source("Source","fit_points",0,3);let(mut b,_)=source("Source","fit_points",3,6);
 if let Json::Object(o)=&mut b{o.insert("input_snapshot_index".into(),Json::Number(2.));}
 let mut v=parse_document(&request(&s,"Circle","fit_batch","Selection",0,[0.;3],None),100_000,10_000).unwrap();
 let Json::Object(o)=&mut v else{panic!()};o.insert("source_refs".into(),Json::Array(vec![a,b]));o.insert("source_snapshots".into(),Json::Array(vec![snapshot.clone(),snapshot]));
 o.insert("points_world".into(),super::input_log::points(&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.],[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]]));
 s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();s.input("").unwrap();
 assert_eq!(rows(get(&evidence(&s),"source_snapshots")).len(),1);
}
#[test]
fn native_point_sources_are_available_to_existing_line_solver_without_changing_geometry(){
 let mut s=CurveSession::default();s.start("Line").unwrap();
 for (index,p)in [[0.,0.,0.],[10.,0.,0.]].into_iter().enumerate(){
  let(r,snapshot)=point_source("Source",index,p);
  let raw=request(&s,"Line","point","Point=Source.Vertex1",0,p,Some((r,snapshot)));
  s.accept_native_json(&raw,u64::MAX).unwrap();
 }
 assert_eq!(s.points(),&[[0.,0.,0.],[10.,0.,0.]]);let v=evidence(&s);assert_eq!(rows(get(&v,"source_refs")).len(),2);
 assert_eq!(rows(get(&v,"accepted_input_events")).len(),2);
}
#[test]
fn native_source_rejection_names_real_command_and_source(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("AroundCurve").unwrap();
 let raw=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0)));
 let bad=raw.replacen("\"ownership\":\"private_native_copy\"","\"ownership\":\"borrowed\"",1);
 let error=s.accept_native_json(&bad,u64::MAX).unwrap_err();assert!(error.starts_with("InvalidBasis:"));assert!(error.contains("Circle")&&error.contains("Source.Edge1"),"{error}");
}
#[test]
fn point_admission_rejects_unrelated_missing_or_empty_source_selection_atomically(){
 let mut s=CurveSession::default();s.start("Line").unwrap();
 for variant in 0..3{
  let(mut r,mut snapshot)=point_source("Source",0,[0.;3]);
  let Json::Object(o)=&mut snapshot else{panic!()};
  o.insert("selected_points_world".into(),Json::Array(vec![point([0.;3])]));
  if variant==1{o.remove("selected_points_world");}
  if variant==2{let Json::Object(o)=&mut r else{panic!()};o.insert("point_end".into(),Json::Number(0.));}
  let raw=request(&s,"Line","point","Point=Source.Vertex1",0,[10.,0.,0.],Some((r,snapshot)));
  assert!(s.accept_native_json(&raw,u64::MAX).is_err(),"variant {variant}");assert!(s.points().is_empty());assert_eq!(rows(&s.accepted_input_log.json()).len(),0);
 }
}
#[test]
fn cad_sources_require_valid_root_location_and_orientation(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("AroundCurve").unwrap();
 for variant in 0..4{
  let(r,mut snapshot)=source("Source","path",0,0);let Json::Object(o)=&mut snapshot else{panic!()};let Json::Object(data)=o.get_mut("geometry_data").unwrap()else{panic!()};
  match variant{0=>{data.remove("native_root_location");},1=>{data.insert("native_root_location".into(),Json::Array(vec![point4([0.;4]);4]));},2=>{data.remove("native_root_orientation");},_=>{data.insert("native_root_orientation".into(),string("unknown"));}}
  let raw=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some((r,snapshot)));
  assert!(s.accept_native_json(&raw,u64::MAX).is_err(),"variant {variant}");assert_eq!(s.circle_reference_mode(),1);
 }
}
fn cloud_source(indices:Option<Vec<usize>>,selected:&[[f64;3]])->(Json,Json){
 let(mut r,mut snapshot)=source("Cloud","fit_points",0,selected.len());
 let Json::Object(o)=&mut r else{panic!()};o.insert("snapshot_frame".into(),string("native_local_with_locations"));o.insert("subelement".into(),string(if indices.is_some(){"Vertex2"}else{""}));
 o.insert("world".into(),super::input_log::vector(&[1.,0.,0.,10.,0.,1.,0.,20.,0.,0.,1.,30.,0.,0.,0.,1.]));
 if let Some(indices)=indices{o.insert("selected_point_indices".into(),Json::Array(indices.into_iter().map(|i|Json::Number(i as f64)).collect()));}
 let Json::Object(o)=&mut snapshot else{panic!()};o.remove("native_shape_type");o.insert("ownership".into(),string("private_plain_copy"));
 o.insert("geometry_data".into(),map([("kind",string("point_cloud")),("points_xyz",super::input_log::points(&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]])),("triangles",Json::Other),("other_native_channels",string("uninspected"))]));
 o.insert("native_parent_to_world".into(),super::input_log::vector(&crate::geometry_frame::IDENTITY));
 o.insert("native_discrete_location".into(),Json::Array(vec![point4([1.,0.,0.,10.]),point4([0.,1.,0.,20.]),point4([0.,0.,1.,30.]),point4([0.,0.,0.,1.])]));
 o.insert("selected_points_world".into(),super::input_log::points(selected));(r,snapshot)
}
#[test]
fn discrete_selection_is_bound_to_full_source_coordinates_and_explicit_subset_indices(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 for indices in [None,Some(vec![0,1,2])]{
  let raw=request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(cloud_source(indices,&[[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]])));
  assert!(s.accept_native_json(&raw,u64::MAX).is_err());assert!(s.points().is_empty());
 }
 // Hand-derived world coordinates: parent identity × translated cloud location.
 let selected=[[9.,20.,30.],[11.,20.,30.],[10.,21.,30.]];
 let mut v=parse_document(&request(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(cloud_source(Some(vec![2,0,1]),&selected))),100_000,10_000).unwrap();
 let Json::Object(o)=&mut v else{panic!()};o.insert("points_world".into(),super::input_log::points(&selected));
 s.accept_native_json(&encode_owned(&v).unwrap(),u64::MAX).unwrap();assert_eq!(s.points(),&selected);
}
#[test]
fn intervening_input_invalidates_same_generation_native_capture(){
 let mut s=CurveSession::default();s.start("Line").unwrap();let(mut r,mut snapshot)=point_source("Source",0,[10.,0.,0.]);
 let Json::Object(o)=&mut snapshot else{panic!()};o.insert("selected_points_world".into(),Json::Array(vec![point([10.,0.,0.])]));
 // Range is deliberately the prospective second input: stale revision must
 // reject even when a later state makes this envelope's range/geometry valid.
 let Json::Object(o)=&mut r else{panic!()};o.insert("point_begin".into(),Json::Number(1.));o.insert("point_end".into(),Json::Number(2.));
 let raw=request(&s,"Line","point","Point=Source.Vertex1",0,[10.,0.,0.],Some((r,snapshot)));
 s.point([0.;3]).unwrap();let log=s.accepted_input_log.json();assert!(s.accept_native_json(&raw,u64::MAX).is_err());assert!(s.active());assert_eq!(s.points(),&[[0.;3]]);assert_eq!(s.accepted_input_log.json(),log);
}
#[test]
fn native_polyline_point_range_above_1024_keeps_supported_solver_capacity(){
 let mut s=CurveSession::default();s.start("Polyline").unwrap();for i in 0..1024{s.point([i as f64,0.,0.]).unwrap();}
 let(r,snapshot)=point_source("Source",1024,[1024.,0.,0.]);
 let raw=request(&s,"Polyline","point","Point=Source.Vertex1",0,[1024.,0.,0.],Some((r,snapshot)));
 s.accept_native_json(&raw,u64::MAX).unwrap();assert_eq!(s.points().len(),1025);assert_eq!(s.points().last(),Some(&[1024.,0.,0.]));
}
#[test]
fn units_in_native_envelope_must_match_captured_solver_scale(){
 let mut s=CurveSession::default();s.start("Line").unwrap();s.set_input_scale(25.4).unwrap();
 let(r,snapshot)=point_source("Source",0,[0.;3]);
 let raw=request(&s,"Line","point","Point=Source.Vertex1",0,[0.;3],Some((r,snapshot)));
 assert!(s.accept_native_json(&raw,u64::MAX).is_err());assert!(s.points().is_empty());
}
#[test]
fn stale_capture_is_rejected_for_every_constructor_and_after_configuration_changes(){
 for name in ["Line","Polyline","Rectangle","InterpCrv","Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(name).unwrap();
  let(r,snapshot)=point_source("Source",1,[10.,0.,0.]);
  let raw=request(&s,name,"point","Point=Source.Vertex1",0,[10.,0.,0.],Some((r,snapshot)));
  s.point([0.;3]).unwrap();let log=s.accepted_input_log.json();assert!(s.accept_native_json(&raw,u64::MAX).is_err(),"{name}");assert_eq!(s.accepted_input_log.json(),log);assert!(s.active());
 }
 for change in 0..2{
  let mut s=CurveSession::default();s.start("Line").unwrap();let(r,snapshot)=point_source("Source",0,[0.;3]);
  let raw=request(&s,"Line","point","Point=Source.Vertex1",0,[0.;3],Some((r,snapshot)));
  if change==0{s.set_input_scale(2.).unwrap();}else{s.set_frame([1.,2.,3.],crate::rectangle::AXES).unwrap();}
  assert!(s.accept_native_json(&raw,u64::MAX).is_err());assert!(s.points().is_empty());
 }
}
#[test]
fn circle_undo_cannot_revalidate_an_old_native_capture_on_the_restored_branch(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("AroundCurve").unwrap();
 let raw=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0)));
 s.accept_native_json(&raw,u64::MAX).unwrap();s.input("Undo").unwrap();assert_eq!(s.circle_reference_mode(),1);
 assert!(s.accept_native_json(&raw,u64::MAX).is_err());
 let fresh=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0)));s.accept_native_json(&fresh,u64::MAX).unwrap();
}
#[test]
fn legacy_mm_native_input_keeps_absent_tolerance_context_without_inventing_one(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("AroundCurve").unwrap();
 let raw=request(&s,"Circle","reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0))).replace(UNITS,"");
 s.accept_native_json(&raw,u64::MAX).unwrap();
 let raw=request(&s,"Circle","reference","OnCurve=0.5",2,[10.,20.,40.],None).replace(UNITS,"");s.accept_native_json(&raw,u64::MAX).unwrap();s.input("2mm").unwrap();
 assert!(s.construction_evidence(u64::MAX).unwrap().contains("\"unit_context_snapshot\":\"\""));
}
