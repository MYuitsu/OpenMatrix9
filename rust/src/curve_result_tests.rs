// SPDX-License-Identifier: LGPL-2.1-or-later
//! Common result schema/replay tests. Native macros prove actual OCCT masters.
use crate::{builder_history::{Json,parse_document},curve::CurveSession,prepared_result::assemble};
#[path="circle_history_role_tests.rs"]
mod history_roles;
fn parse(raw:&str)->Json{parse_document(raw,16*1024*1024,1_000_000).unwrap()}
fn get<'a>(v:&'a Json,key:&str)->&'a Json{let Json::Object(o)=v else{panic!("object")};o.get(key).unwrap_or_else(||panic!("missing {key}"))}
fn rows(v:&Json)->&[Json]{let Json::Array(a)=v else{panic!("array")};a}
fn request(construction:&str,units:&str)->String{
 format!(r#"{{"snapshot":{{"schema_version":1,"native_shape_type":5,"ownership":"private_native_copy","unit_context_snapshot":"{units}","geometry_data":{{"representation":"polycurve","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 unit output fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":false,"closed":false}}}},"source_attributes":{{"schema_version":1,"availability":"not_applicable_constructed_output"}}}},"validation":{{"native_valid":true,"valid_solid":false,"closed":false}},"input_identity":{{"document_name":"fixture","name":"Curve"}},"operation_context":{{"schema_version":1,"operation":"ConstructCurve","construction":{construction},"output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"source_evidence_in_input_snapshots_output_attributes_unavailable"}}}}"#)
}
fn construction(command:&str,inputs:&[&str])->String{
 let mut s=CurveSession::default();s.start(command).unwrap();for input in inputs{s.input(input).unwrap();}s.construction_evidence(u64::MAX).unwrap()
}
#[test]
fn all_six_existing_constructors_publish_common_results_with_original_points(){
 let cases=[("Line",vec!["BothSides=Yes","0,0,0","10,0,0"],"[[0,0,0],[10,0,0]]"),
 ("Polyline",vec!["0,0,0","3,0,0","3,4,0","Close"],"[[0,0,0],[3,0,0],[3,4,0]]"),
 ("Rectangle",vec!["0,0,0","4,3,0"],"[[0,0,0],[4,3,0]]"),
 ("Circle",vec!["3Point","1,0,0","0,1,0","-1,0,0"],"[[1,0,0],[0,1,0],[-1,0,0]]"),
 ("Ellipse",vec!["1,2,3","6,2,3","1,5,3"],"[[1,2,3],[6,2,3],[1,5,3]]"),
 ("InterpCrv",vec!["Degree=1","0,0,0","3,0,0","3,4,0",""],"[[0,0,0],[3,0,0],[3,4,0]]")];
 for(command,inputs,expected)in cases{
  let recipe=construction(command,&inputs);let raw=assemble(&request(&recipe,"")).expect(command);let result=parse(&raw);
  assert_eq!(get(&result,"operation"),&Json::String("ConstructCurve".into()));
  assert_eq!(get(get(&result,"operation_context"),"construction"),&parse(&recipe));
  let snapshots=rows(get(&result,"input_snapshots"));assert_eq!(snapshots.len(),1);
  assert_eq!(get(get(&snapshots[0],"geometry_data"),"points_xyz"),&parse(expected));
  assert_eq!(rows(get(&result,"output_snapshots")).len(),1);
  assert!(raw.contains("native_curve_construction")&&!raw.contains("NativeBuilder")&&!raw.contains("upstream_archive_conversion"));
  assert_eq!(get(&result,"topology_map"),&Json::Other);
 }
}
#[test]
fn result_rejects_corrupt_options_basis_points_frame_and_output_identity(){
 let recipe=construction("Line",&["BothSides=Yes","0,0,0","10,0,0"]);let raw=request(&recipe,"");
 assert!(assemble(&raw).is_ok());
 for bad in [raw.replace("\"both_sides\":true","\"both_sides\":false"),raw.replace("\"normalized_points\":[[0,0,0],[10,0,0]]","\"normalized_points\":[]"),
 raw.replace("\"input_mm_per_unit\":1","\"input_mm_per_unit\":2"),raw.replace("\"generation\":\"18446744073709551615\"","\"generation\":\"0\""),
 raw.replace("\"native_shape_type\":5","\"native_shape_type\":2"),raw.replace("\"native_valid\":true","\"native_valid\":false")]{
  assert_ne!(bad,raw,"mutation must change the real request");assert!(assemble(&bad).is_err());
 }
 let recipe=construction("InterpCrv",&["Degree=1","0,0,0","3,0,0","3,4,0",""]);
 let bad=recipe.replace("\"degree\":1","\"degree\":2");assert_ne!(bad,recipe);assert!(assemble(&request(&bad,"")).is_err());
}
#[test]
fn mouse_ortho_result_retains_the_original_probe_and_replays_normalized_line(){
 let mut s=CurveSession::default();s.start("Line").unwrap();s.mouse_point([0.;3],false).unwrap();s.mouse_point([2.,1.,0.],true).unwrap();
 let raw=assemble(&request(&s.construction_evidence(9).unwrap(),"")).unwrap();let result=parse(&raw);
 assert_eq!(get(get(&rows(get(&result,"input_snapshots"))[0],"geometry_data"),"points_xyz"),&parse("[[0,0,0],[2,1,0]]"));
 assert_eq!(get(get(get(&result,"operation_context"),"construction"),"normalized_points"),&parse("[[0,0,0],[2,0,0]]"));
}
#[test]
fn numeric_quantity_events_remain_owned_with_explicit_units_and_no_invented_points(){
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.set_input_scale(25.4).unwrap();s.input("1,2,3").unwrap();s.input("10mm").unwrap();
 let recipe=s.construction_evidence(10).unwrap();let units="1|1|2.54e1|1e0|1e-2|1e-3|1e-2";
 let raw=assemble(&request(&recipe,units)).unwrap();let result=parse(&raw);let numeric=&rows(get(&result,"input_snapshots"))[0];
 assert_eq!(get(get(numeric,"geometry_data"),"points_xyz"),&parse("[[25.4,50.8,76.19999999999999]]"));
 assert_eq!(get(numeric,"unit_context_revision"),&Json::String("1".into()));assert!(raw.contains("10mm"));
 assert!(assemble(&request(&recipe,"")).is_err());
}
#[test]
fn result_replays_native_path_sources_and_resolves_common_source_indices(){
 use super::native_input_tests::{request as native,source};
 let units="1|1|1e0|1e0|1e-2|1e-3|1e-2";
 for command in ["Circle","Ellipse"]{
  let mut s=CurveSession::default();s.start(command).unwrap();s.input("AroundCurve").unwrap();
  s.accept_native_json(&native(&s,command,"reference","Curve=Source.Edge1",1,[0.;3],Some(source("Source","path",0,0))),u64::MAX).unwrap();
  s.accept_native_json(&native(&s,command,"reference","OnCurve=0.2500",2,[10.,20.,35.],None),u64::MAX).unwrap();
  s.input("5mm").unwrap();if command=="Ellipse"{s.input("3mm").unwrap();}
  let recipe=s.construction_evidence(u64::MAX).unwrap();let result=parse(&assemble(&request(&recipe,units)).unwrap());
  let inputs=rows(get(&result,"input_snapshots"));let refs=rows(get(get(&result,"operation_context"),"source_refs"));
  assert_eq!(inputs.len(),2);assert_eq!(refs.len(),1);assert_eq!(get(&refs[0],"input_snapshot_index"),&Json::Number(1.));
  assert_eq!(get(get(&inputs[1],"source_attributes"),"object_name"),&Json::String("Source".into()));
  assert_eq!(get(get(&inputs[0],"geometry_data"),"points_xyz"),&parse("[[10,20,35]]"));
  let corrupt=recipe.replacen("\"input_snapshot_index\":1","\"input_snapshot_index\":0",1);
  assert_ne!(corrupt,recipe);assert!(assemble(&request(&corrupt,units)).is_err());
 }
}
#[test]
fn result_replays_fit_undo_branch_without_removed_native_sources(){
 use super::native_input_tests::{request as native,source};
 let mut s=CurveSession::default();s.start("Circle").unwrap();s.input("FitPoints").unwrap();
 for name in ["RemovedCloud","AcceptedCloud"]{
  s.accept_native_json(&native(&s,"Circle","fit_batch","Selection",0,[0.;3],Some(source(name,"fit_points",0,3))),u64::MAX).unwrap();
  if name=="RemovedCloud"{s.input("Undo").unwrap();}
 }
 s.input("").unwrap();let raw=assemble(&request(&s.construction_evidence(u64::MAX).unwrap(),"1|1|1e0|1e0|1e-2|1e-3|1e-2")).unwrap();
 assert!(!raw.contains("RemovedCloud"));let result=parse(&raw);assert_eq!(rows(get(&result,"input_snapshots")).len(),2);
 assert_eq!(get(get(&rows(get(&result,"input_snapshots"))[0],"geometry_data"),"points_xyz"),&parse("[[1,0,0],[0,1,0],[-1,0,0]]"));
}
fn focus_request(role:&str,position:&str)->String{
 let recipe=construction("Ellipse",&["FromFoci","MarkFoci=Yes","6,20,30","14,20,30","10,23,30"]);
 let mut value=parse(&request(&recipe,""));let Json::Object(ref mut r)=value else{unreachable!()};
 let Json::Object(c)=r.get_mut("operation_context").unwrap()else{unreachable!()};c.insert("output_role".into(),Json::String(role.into()));
 let Json::Object(s)=r.get_mut("snapshot").unwrap()else{unreachable!()};s.insert("native_shape_type".into(),Json::Number(7.));
 let Json::Object(g)=s.get_mut("geometry_data").unwrap()else{unreachable!()};g.insert("representation".into(),Json::String("point".into()));
 g.insert("native_root_location".into(),parse("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]"));
 let Json::Object(t)=s.get_mut("topology_data").unwrap()else{unreachable!()};
 t.insert("vertices".into(),parse(&format!(r#"[{{"vertex_id":"1","point_xyz":{position},"tolerance":1e-7,"geometry_coordinates":"native_local","native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}}]"#)));
 crate::prepared_result::encode_owned(&value).unwrap()
}

fn history_request()->String{
 // Typed numeric input has no source role and therefore stays unchanged.
 let original=construction("Circle",&["6,8,10","Radius=2"]);let mut value=parse(&request(&original,""));
 let Json::Object(r)=&mut value else{unreachable!()};
 r.insert("operation_context".into(),parse(&format!(r#"{{"schema_version":1,"operation":"ReplayCircleHistory","feature_id":"OM9-CURVE-005","storage":"recipe_v2","recipe":[2,0,0,2,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0,3,8,0,-1,0],"points_world":[[6,8,10]],"native_plan":null,"resolved_plan":{{"center":[6,8,10],"normal":[0,0,1],"radius":2,"fit_deviation":0,"approx_deviation":0}},"original_construction":{},"source_refs":[],"source_snapshots":[],"source_snapshot_indexing":"input_snapshots_zero_based_including_numeric_input","output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"source_evidence_in_input_snapshots_output_attributes_unavailable"}}"#,crate::prepared_result::encode_owned(&Json::String(original)).unwrap())));
 let Json::Object(c)=r.get_mut("operation_context").unwrap()else{unreachable!()};
 c.insert("recorded_recipe".into(),c["recipe"].clone());c.insert("recorded_points_world".into(),c["points_world"].clone());
 c.insert("recorded_source_roles".into(),parse("[]"));c.insert("recorded_source_parameters".into(),parse("[]"));
 let Json::Object(s)=r.get_mut("snapshot").unwrap()else{unreachable!()};let Json::Object(t)=s.get_mut("topology_data").unwrap()else{unreachable!()};
 t.insert("edges_3d".into(),parse(r#"[{"domain":[0,6.283185307179586],"start_vertex_id":"1","end_vertex_id":"1","degenerate":false,"curve_present":true,"geometry_data":{"kind":"analytic_curve","analytic_type":"circle","origin_xyz":[6,8,10],"axes_xyz":[[1,0,0],[0,1,0],[0,0,1]],"parameters":{"radius":2}},"native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}]"#));
 t.insert("vertices".into(),parse(r#"[{"vertex_id":"1","point_xyz":[8,8,10],"tolerance":1e-7,"geometry_coordinates":"native_local","native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]}]"#));
 crate::prepared_result::encode_owned(&value).unwrap()
}
#[test]
fn history_replay_result_retains_original_construction_and_current_points(){
 let result=parse(&assemble(&history_request()).expect("owned current Circle History result"));
 assert_eq!(get(&result,"operation"),&Json::String("ReplayCircleHistory".into()));
 let context=get(&result,"operation_context");assert_eq!(get(context,"storage"),&Json::String("recipe_v2".into()));
 let original=match get(context,"original_construction"){Json::String(raw)=>parse(raw),_=>panic!("original")};
 assert_eq!(get(&original,"normalized_points"),&parse("[[6,8,10]]"));
 assert_eq!(get(get(&rows(get(&result,"input_snapshots"))[0],"geometry_data"),"points_xyz"),&parse("[[6,8,10]]"));
 assert!(rows(get(&result,"conversion_reports")).iter().any(|v|get(v,"phase")==&Json::String("native_circle_history_replay".into())));
}
#[test]
fn history_replay_result_rejects_asserted_plan_geometry_and_recipe_corruption(){
 let raw=history_request();assert!(assemble(&raw).is_ok());
 for bad in [raw.replacen("\"radius\":2","\"radius\":7",1),raw.replacen("\"points_world\":[[6,8,10]]","\"points_world\":[[9,8,10]]",1),
 raw.replacen("\"recipe\":[2,0,0,2","\"recipe\":[2,0,0,5",1),raw.replacen("\"storage\":\"recipe_v2\"","\"storage\":\"unknown\"",1),
 raw.replacen("\"origin_xyz\":[6,8,10]","\"origin_xyz\":[12,8,10]",1)]{
  assert_ne!(bad,raw);assert!(assemble(&bad).is_err());
 }
}
#[test]
fn history_replay_result_admits_explicit_legacy_path_storage_without_fabricated_events(){
 let mut value=parse(&history_request());let Json::Object(r)=&mut value else{unreachable!()};let Json::Object(c)=r.get_mut("operation_context").unwrap()else{unreachable!()};
 c.insert("storage".into(),Json::String("legacy_around_curve".into()));c.insert("original_construction".into(),Json::String(String::new()));
 c.insert("recorded_recipe".into(),parse("[]"));c.insert("recorded_points_world".into(),parse("[]"));
 c.insert("recipe".into(),parse("[2,4,0,2,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0,3,8,0,-1,0]"));
 c.insert("native_plan".into(),parse("[6,8,10,0,0,1,1e-7]"));
 let(mut reference,mut snapshot)=super::native_input_tests::source("Source","history_path",0,0);
 if let Json::Object(r)=&mut reference{r.insert("role_index".into(),Json::Number(0.));r.insert("selected_representation".into(),Json::Number(2.));}
 if let Json::Object(s)=&mut snapshot{if let Json::Object(t)=s.get_mut("topology_data").unwrap(){t.insert("edges_3d".into(),parse(r#"[{"native_topology_index":"1","domain":[0,2],"curve_present":true,"degenerate":false,"geometry_coordinates":"native_local","native_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"geometry_data":{"schema_version":1,"kind":"analytic_curve","analytic_type":"line","dimension":3,"parameter_convention":"occt_native","origin_xyz":[6,8,9],"axes_xyz":[[0,0,1],[0,0,0],[0,0,0]],"parameters":{},"natural_domains":["unbounded"]}}]"#));}}
 c.insert("path_fraction".into(),Json::Number(0.5));
 c.insert("source_refs".into(),Json::Array(vec![reference]));c.insert("source_snapshots".into(),Json::Array(vec![snapshot]));
 let Json::Object(s)=r.get_mut("snapshot").unwrap()else{unreachable!()};s.insert("unit_context_snapshot".into(),Json::String("1|1|1e0|1e0|1e-2|1e-3|1e-2".into()));
 let result=parse(&assemble(&crate::prepared_result::encode_owned(&value).unwrap()).expect("legacy owned replay result"));
 assert_eq!(get(get(&result,"operation_context"),"original_construction"),&Json::String(String::new()));
 assert_eq!(get(get(get(&rows(get(&result,"input_snapshots"))[0],"geometry_data"),"history_storage"),"original_command_events"),&Json::String("unavailable_legacy_record".into()));
}
#[test]
fn history_native_plan_requires_owned_matching_path_source(){
 let mut value=parse(&history_request());let Json::Object(r)=&mut value else{unreachable!()};let Json::Object(c)=r.get_mut("operation_context").unwrap()else{unreachable!()};
 c.insert("storage".into(),Json::String("legacy_around_curve".into()));c.insert("original_construction".into(),Json::String(String::new()));
 c.insert("recipe".into(),parse("[2,4,0,2,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0,3,8,0,-1,0]"));c.insert("native_plan".into(),parse("[6,8,10,0,0,1,1e-7]"));
 let unowned=crate::prepared_result::encode_owned(&value).unwrap();assert!(assemble(&unowned).is_err(),"native solve evidence requires a full source");
}
#[test]
fn history_result_rejects_trimmed_circle_and_open_endpoint_topology(){
 let raw=history_request();assert!(assemble(&raw).is_ok());
 for bad in [raw.replace("\"domain\":[0,6.283185307179586]","\"domain\":[0,3.141592653589793]"),
 raw.replace("\"end_vertex_id\":\"1\"","\"end_vertex_id\":\"2\""),
 raw.replace("\"degenerate\":false","\"degenerate\":true")]{
  assert_ne!(bad,raw);assert!(assemble(&bad).is_err(),"partial/open native support cannot be published as a whole Circle");
 }
}

// Real authored native snapshots, reconstructed from saved Runtime59 results.
// These tests do not claim intercepted requests or independent BREP decoding.
fn captured_history_request(name:&str)->Json{
 parse(match name{
  "around"=>include_str!("../tests/fixtures/circle_history_native_source/contact_replay/around.json"),
  "tangent"=>include_str!("../tests/fixtures/circle_history_native_source/contact_replay/tangent.json"),
  "fixed_contact"=>include_str!("../tests/fixtures/circle_history_native_source/contact_replay/fixed_contact.json"),
  _=>panic!("unknown captured History case"),
 })
}
fn captured_history_assemble(value:&Json)->Result<String,u32>{
 assemble(&crate::prepared_result::encode_owned(value).unwrap())
}
fn captured_history_context(value:&mut Json)->&mut std::collections::BTreeMap<String,Json>{
 let Json::Object(root)=value else{panic!("request")};
 let Json::Object(context)=root.get_mut("operation_context").unwrap()else{panic!("context")};context
}
#[test]
fn history_native_source_actual_around_request_is_accepted(){
 let v=captured_history_request("around");assert!(captured_history_assemble(&v).is_ok());
 assert_eq!(get(get(&v,"operation_context"),"native_plan"),&parse("[10,0,0,1,0,0,1e-7]"));
}
#[test]
fn history_native_source_around_rejects_wrong_owned_tangent_normal(){
 let mut v=captured_history_request("around");let c=captured_history_context(&mut v);
 c.insert("native_plan".into(),parse("[10,0,0,-1,0,0,1e-7]"));
 let Json::Object(plan)=c.get_mut("resolved_plan").unwrap()else{panic!("plan")};
 plan.insert("normal".into(),parse("[-1,0,0]"));
 let Json::Object(root)=&v else{panic!("root")};
 assert!(crate::circle_history::result::validate(crate::prepared_result::object(get(&v,"operation_context")).unwrap(),crate::prepared_result::object(root.get("snapshot").unwrap()).unwrap()).is_err(),"consistent replay normal cannot replace the actual path tangent");
}
#[test]
fn history_native_source_tangent_fraction_must_bind_the_recorded_probe(){
 let mut v=captured_history_request("tangent");captured_history_context(&mut v).insert("recorded_source_parameters".into(),parse("[0.5,0.6]"));
 assert!(captured_history_assemble(&v).is_err(),"fraction.5 gives world(1,0,0), not probe(1,4,0)");
}
#[test]
fn history_native_source_fixed_point_binds_source_even_when_on_same_circle(){
 let mut v=captured_history_request("fixed_contact");captured_history_context(&mut v).insert("points_world".into(),parse("[[1,4,0],[5,0,0]]"));
 assert!(captured_history_assemble(&v).is_err(),"another valid circumference point cannot stand in for captured Vertex1(5,8,0)");
}
#[test]
fn history_native_source_actual_tangent_request_is_accepted(){
 assert!(captured_history_assemble(&captured_history_request("tangent")).is_ok());
}
#[test]
fn history_native_source_actual_fixed_contact_request_is_accepted(){
 assert!(captured_history_assemble(&captured_history_request("fixed_contact")).is_ok());
}
#[test]
fn history_native_source_around_fraction_must_match_current_plan(){
 let mut v=captured_history_request("around");captured_history_context(&mut v).insert("path_fraction".into(),Json::Number(0.25));
 assert!(captured_history_assemble(&v).is_err(),"line domain0..20 at fraction.25 is(5,0,0), not the old plan center(10,0,0)");
}
#[test]
fn history_native_source_around_requires_an_explicit_path_role(){
 let mut v=captured_history_request("around");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};
 let Json::Object(r)=&mut refs[0]else{panic!("ref")};r.insert("role".into(),Json::String("history_source".into()));
 assert!(captured_history_assemble(&v).is_err(),"an owned arbitrary source must not stand in for the missing History path");
}
#[test]
fn history_native_source_around_rejects_ambiguous_duplicate_path_roles(){
 let mut v=captured_history_request("around");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs.push(refs[0].clone());
 assert!(captured_history_assemble(&v).is_err(),"one SourceCurve cannot bind two indistinguishable path roles");
}
#[test]
fn history_native_source_tangent_points_must_correspond_to_owned_curve(){
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 c.insert("points_world".into(),parse("[[100,4,0],[4,0,0]]"));
 assert!(captured_history_assemble(&v).is_err(),"the first claimed current point is off the captured vertical line x=1");
}
#[test]
fn history_native_source_tangent_plan_must_bind_selected_source_geometry(){
 let replacement=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/replacement_line.json"));
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs[0]=get(&replacement,"reference").clone();
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};snapshots[0]=get(&replacement,"snapshot").clone();
 assert!(captured_history_assemble(&v).is_err(),"honestly owned horizontal source y=0 does not support the stale vertical contact(1,4,0)");
}
#[test]
fn history_native_source_rejects_mixed_document_tokens(){
 let replacement=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/replacement_point.json"));
 let mut v=captured_history_request("fixed_contact");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs[0]=get(&replacement,"reference").clone();
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};snapshots[0]=get(&replacement,"snapshot").clone();
 let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};points[1]=get(&replacement,"point").clone();
 assert_eq!(captured_history_assemble(&v),Err(1),"same document UID does not erase a mismatch between native document tokens2 and1");
}
#[test]
fn history_native_source_fixed_point_must_lie_on_current_tangent_circle(){
 let replacement=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/replacement_point_same_document.json"));
 let mut v=captured_history_request("fixed_contact");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs[0]=get(&replacement,"reference").clone();
 assert_eq!(get(&refs[0],"document"),get(&refs[1],"document"));
 assert_eq!(get(&refs[0],"document_uid"),get(&refs[1],"document_uid"));
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};snapshots[0]=get(&replacement,"snapshot").clone();
 let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};points[1]=get(&replacement,"point").clone();
 assert!(captured_history_assemble(&v).is_err(),"the genuine same-document fixed point(0,10,0) is off the stale circle center(5,4,0),radius4");
}
#[test]
fn history_native_source_rejects_unbound_role_index(){
 let mut v=captured_history_request("around");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};
 let Json::Object(reference)=&mut refs[0]else{panic!("reference")};
 reference.insert("role_index".into(),Json::Number(1.));
 assert!(captured_history_assemble(&v).is_err(),"a sole path cannot claim a nonexistent second source role");
}
#[test]
fn history_native_source_tangent_rejects_duplicate_role_indices(){
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};
 let Json::Object(reference)=&mut refs[1]else{panic!("reference")};
 reference.insert("role_index".into(),Json::Number(0.));
 assert!(captured_history_assemble(&v).is_err(),"two owned sources cannot both stand for the first recorded role");
}
#[test]
fn history_native_source_tangent_requires_complete_recorded_roles(){
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 c.insert("recorded_source_roles".into(),parse("[2,0,1]"));
 assert!(captured_history_assemble(&v).is_err(),"every owned tangent source needs its recorded role and point span");
}
#[test]
fn history_native_source_tangent_rejects_duplicate_constraint_spans(){
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 c.insert("recorded_source_roles".into(),parse("[2,0,1,2,0,1]"));
 assert!(captured_history_assemble(&v).is_err(),"two curves cannot silently overwrite one tangent constraint");
}
#[test]
fn history_native_source_tangent_rejects_overlapping_point_and_curve_constraints(){
 let mut v=captured_history_request("fixed_contact");let c=captured_history_context(&mut v);
 c.insert("recorded_source_roles".into(),parse("[0,0,1,2,0,1]"));
 assert!(captured_history_assemble(&v).is_err(),"a curve role cannot overwrite a fixed point at the same recorded constraint offset");
}
#[test]
fn history_native_source_fixed_point_must_share_current_tangent_plane(){
 let mut v=captured_history_request("fixed_contact");let c=captured_history_context(&mut v);
 let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};
 // Radius remains exactly4 from center(5,4,0), but the height above XY is4.
 // This is an intentionally altered numeric request, not a new native capture.
 points[1]=parse("[5,4,4]");
 assert!(captured_history_assemble(&v).is_err(),"radial distance alone cannot certify a fixed point in the circle plane");
}
#[test]
fn history_native_source_fixed_point_preserves_native_roundoff_tolerance(){
 let mut v=captured_history_request("fixed_contact");let c=captured_history_context(&mut v);
 let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};
 // Synthetic numeric perturbation within the native Tangent solver's accepted
 // radial error. It is not a claim that the captured source moved this way.
 points[1]=parse("[5,8.0000005,0]");
 assert!(captured_history_assemble(&v).is_ok(),"the independent point check must preserve accepted native roundoff");
}
#[test]
fn history_native_source_tangent_curve_role_cannot_bind_a_point_source(){
 let replacement=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/replacement_point_same_document.json"));
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs[0]=get(&replacement,"reference").clone();
 let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};snapshots[0]=get(&replacement,"snapshot").clone();
 let Json::Array(points)=c.get_mut("points_world").unwrap()else{panic!("points")};points[0]=get(&replacement,"point").clone();
 assert!(captured_history_assemble(&v).is_err(),"a genuinely owned Point does not satisfy a recorded native curve constraint");
}
#[test]
fn history_native_source_all_twenty_actual_replay_modes_remain_supported(){
 let cases=parse(include_str!("../tests/fixtures/circle_history_native_source/contact_replay/all_modes.json"));
 assert_eq!(rows(&cases).len(),20);
 for case in rows(&cases){
  assert!(captured_history_assemble(get(case,"request")).is_ok(),"reconstructed authored native replay {}",crate::prepared_result::encode_owned(get(case,"name")).unwrap());
 }
}
#[test]
fn history_native_source_around_distinguishes_recorded_probe_from_resolved_anchor(){
 let v=captured_history_request("around");let result=parse(&captured_history_assemble(&v).unwrap());
 let geometry=get(&rows(get(&result,"input_snapshots"))[0],"geometry_data");
 assert_eq!(get(geometry,"role"),&Json::String("circle_history_replay_inputs".into()));
 assert_eq!(get(geometry,"points_xyz"),&parse("[[5,0,0]]"));
 assert_eq!(get(geometry,"point_roles"),&parse("[\"recorded_path_probe\"]"));
 assert_eq!(get(geometry,"resolved_path_anchor_xyz"),&parse("[10,0,0]"));
 assert_eq!(get(get(&result,"operation_context"),"recorded_points_world"),&parse("[[5,0,0]]"));
}
#[test]
fn history_owned_source_admission_matches_all_nineteen_native_curve_families(){
 let cases=parse(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"));let mut queries=0;
 for case in rows(&cases){for query in rows(get(case,"queries")){
  let mut v=captured_history_request("around");let c=captured_history_context(&mut v);
  let Json::Array(recipe)=c.get_mut("recipe").unwrap()else{panic!("recipe")};recipe[18]=Json::Number(0.);
  c.insert("recorded_recipe".into(),c["recipe"].clone());
  let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
  let Json::Object(snapshot)=&mut snapshots[0]else{panic!("snapshot")};
  let Json::Object(topology)=snapshot.get_mut("topology_data").unwrap()else{panic!("topology")};
  let Json::Array(edges)=topology.get_mut("edges_3d").unwrap()else{panic!("edges")};
  let Json::Object(edge)=&mut edges[0]else{panic!("edge")};
  edge.insert("geometry_data".into(),get(case,"geometry_data").clone());edge.insert("domain".into(),get(case,"domain").clone());
  let domain=rows(get(case,"domain"));let p=crate::prepared_result::array::<3>(get(query,"value")).unwrap();
  let d=crate::prepared_result::array::<3>(get(query,"derivative")).unwrap();let length=d[0].hypot(d[1]).hypot(d[2]);let d=d.map(|n|n/length);
  let num=|v:&Json|crate::prepared_result::number(v).unwrap();
  c.insert("path_fraction".into(),Json::Number((num(get(query,"parameter"))-num(&domain[0]))/(num(&domain[1])-num(&domain[0]))));
  c.insert("native_plan".into(),Json::Array(p.into_iter().chain(d).chain([1e-7]).map(Json::Number).collect()));
  // Existing replay normalizes the native plan, then the numeric-radius plan.
  // Prepare that envelope independently from the new source evaluator.
  let normal=crate::circle::Plan::new(p,crate::circle::Plan::new(p,d,1e-7).unwrap().normal,4.).unwrap().normal;
  c.insert("resolved_plan".into(),crate::prepared_result::map([("center",Json::Array(p.map(Json::Number).into())),("normal",Json::Array(normal.map(Json::Number).into())),("radius",Json::Number(4.)),("fit_deviation",Json::Number(0.)),("approx_deviation",Json::Number(0.))]));
  let context=crate::prepared_result::object(get(&v,"operation_context")).unwrap();let output=crate::prepared_result::object(get(&v,"snapshot")).unwrap();
  // This exercises source admission, not output publication: the altered source
  // and numeric plan are synthetic around actual native value/D1 oracle rows.
  let admission=crate::circle_history::result::validate(context,output);
  assert!(admission.is_ok(),"owned family {} parameter {:?}: {:?}",crate::prepared_result::encode_owned(get(case,"name")).unwrap(),get(query,"parameter"),admission.err());queries+=1;
 }}
 assert_eq!(queries,183);
}
#[test]
fn history_tangent_contact_must_bind_plan_even_when_all_probes_match(){
 let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
 // Explicit synthetic contact evidence, independently derived on these lines:
 // first contact(1,4,0) at.6, second(5,0,0) at.625. Probe2 remains(4,0,0).
 c.insert("resolved_contact_fractions".into(),parse("[0.6,0.625]"));
 c.insert("resolved_contact_indexing".into(),Json::String("current_constraint_points_zero_based".into()));
 c.insert("native_plan".into(),parse("[5,5,0,0,0,1,4]"));
 let Json::Object(plan)=c.get_mut("resolved_plan").unwrap()else{panic!("plan")};plan.insert("center".into(),parse("[5,5,0]"));
 let context=crate::prepared_result::object(get(&v,"operation_context")).unwrap();let output=crate::prepared_result::object(get(&v,"snapshot")).unwrap();
 assert!(crate::circle_history::result::validate(context,output).is_err(),"probes still match x=1 and y=0, but center(5,5),radius4 is not tangent to y=0");
}
fn synthetic_history_contact_request(name:&str)->Json{
 let mut v=captured_history_request(name);let c=captured_history_context(&mut v);
 c.insert("resolved_contact_fractions".into(),parse(if name=="tangent"{"[0.6,0.625]"}else{"[0.6,null]"}));
 c.insert("resolved_contact_indexing".into(),Json::String("current_constraint_points_zero_based".into()));v
}
#[test]
fn history_tangent_contacts_preserve_distinct_probe_and_fixed_first_semantics(){
 for name in ["tangent","fixed_contact"]{
  // Analytically derived fractions on actual owned lines; this is synthetic
  // metadata, not a claim that the old native producer recorded contacts.
  assert!(captured_history_assemble(&synthetic_history_contact_request(name)).is_ok());
 }
}
#[test]
fn history_tangent_contacts_reject_missing_wrong_fraction_and_point_metadata(){
 for(name,contacts)in [("tangent","[0.6,0.6]"),("tangent","[0.6,1.01]"),("tangent","[null,0.625]"),("tangent","[0.6]"),("fixed_contact","[0.6,0.5]")]{
  let mut v=synthetic_history_contact_request(name);captured_history_context(&mut v).insert("resolved_contact_fractions".into(),parse(contacts));
  assert!(captured_history_assemble(&v).is_err(),"wrong contact evidence {name} {contacts}");
 }
 let mut v=synthetic_history_contact_request("tangent");captured_history_context(&mut v).remove("resolved_contact_fractions");
 assert!(captured_history_assemble(&v).is_err(),"a current tangent request cannot silently skip contact verification");
}
#[test]
fn history_tangent_contacts_reject_wrong_plane_direction_and_changed_first_contact(){
 for(kind,data,probe_fraction,contact_fraction)in [
  ("wrong tangent plane",r#"{"schema_version":1,"kind":"analytic_curve","analytic_type":"circle","dimension":3,"parameter_convention":"occt_native","origin_xyz":[5,4,0],"axes_xyz":[[-1,0,0],[0,0,1],[0,1,0]],"parameters":{"radius":4},"natural_domains":[[0,6.283185307179586]]}"#,0.,0.),
  ("wrong radial tangent",r#"{"schema_version":1,"kind":"analytic_curve","analytic_type":"line","dimension":3,"parameter_convention":"occt_native","origin_xyz":[0,4,0],"axes_xyz":[[0.7071067811865476,0.7071067811865476,0],[0,0,0],[0,0,0]],"parameters":{},"natural_domains":["unbounded"]}"#,0.,0.),
  ("changed first contact",r#"{"schema_version":1,"kind":"analytic_curve","analytic_type":"circle","dimension":3,"parameter_convention":"occt_native","origin_xyz":[4,4,0],"axes_xyz":[[1,0,0],[0,1,0],[0,0,1]],"parameters":{"radius":4},"natural_domains":[[0,6.283185307179586]]}"#,0.5,0.75),
 ]{
  let mut v=synthetic_history_contact_request("fixed_contact");let c=captured_history_context(&mut v);
  let Json::Array(parameters)=c.get_mut("recorded_source_parameters").unwrap()else{panic!("params")};parameters[1]=Json::Number(probe_fraction);
  let Json::Array(contacts)=c.get_mut("resolved_contact_fractions").unwrap()else{panic!("contacts")};contacts[0]=Json::Number(contact_fraction);
  let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};
  let Json::Object(snapshot)=&mut snapshots[1]else{panic!("snapshot")};let Json::Object(topology)=snapshot.get_mut("topology_data").unwrap()else{panic!("topology")};
  let Json::Array(edges)=topology.get_mut("edges_3d").unwrap()else{panic!("edges")};let Json::Object(edge)=&mut edges[0]else{panic!("edge")};
  // These analytic replacements are intentionally synthetic. The line location
  // already translates by x=1; circle origins account for that same placement.
  let mut geometry=parse(data);if kind=="wrong tangent plane"{let Json::Object(g)=&mut geometry else{panic!("g")};g.insert("origin_xyz".into(),parse("[4,4,0]"));}
  edge.insert("geometry_data".into(),geometry);edge.insert("domain".into(),parse(if kind=="wrong radial tangent"{"[0,40]"}else{"[0,6.283185307179586]"}));
  assert!(captured_history_assemble(&v).is_err(),"{kind}");
 }
}
#[test]
fn history_tangent_contact_admission_matches_nineteen_native_curve_families(){
 let cases=parse(include_str!("../tests/fixtures/owned_curve_evaluation/native_jets_oracle.json"));let mut count=0;
 let xyz=|v:&Json|crate::prepared_result::array::<3>(v).unwrap();
 let cross=|a:[f64;3],b:[f64;3]|[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
 let unit=|p:[f64;3]|{let n=p[0].hypot(p[1]).hypot(p[2]);p.map(|v|v/n)};
 for case in rows(&cases){for query in rows(get(case,"queries")){
  let p=xyz(get(query,"value"));let t=unit(xyz(get(query,"derivative")));
  let radial=unit(cross(t,if t[2].abs()<0.9{[0.,0.,1.]}else{[1.,0.,0.]}));let normal=unit(cross(t,radial));
  let center:[f64;3]=std::array::from_fn(|j|p[j]+4.*radial[j]);let fixed:[f64;3]=std::array::from_fn(|j|center[j]+4.*radial[j]);
  let domain=crate::prepared_result::array::<2>(get(case,"domain")).unwrap();let fraction=(crate::prepared_result::number(get(query,"parameter")).unwrap()-domain[0])/(domain[1]-domain[0]);
  let mut v=captured_history_request("tangent");let c=captured_history_context(&mut v);
  let Json::Array(refs)=c.get_mut("source_refs").unwrap()else{panic!("refs")};refs.truncate(1);
  let Json::Array(snapshots)=c.get_mut("source_snapshots").unwrap()else{panic!("snapshots")};snapshots.truncate(1);
  let Json::Object(snapshot)=&mut snapshots[0]else{panic!("snapshot")};let Json::Object(topology)=snapshot.get_mut("topology_data").unwrap()else{panic!("topology")};
  let Json::Array(edges)=topology.get_mut("edges_3d").unwrap()else{panic!("edges")};let Json::Object(edge)=&mut edges[0]else{panic!("edge")};
  edge.insert("geometry_data".into(),get(case,"geometry_data").clone());edge.insert("domain".into(),get(case,"domain").clone());
  edge.insert("native_location".into(),parse("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]"));
  c.insert("points_world".into(),Json::Array(vec![Json::Array(p.map(Json::Number).into()),Json::Array(fixed.map(Json::Number).into())]));
  // The second constraint is a typed fixed point, not a native source. Its
  // recorded value must be this independently constructed oracle point.
  c.insert("recorded_points_world".into(),c["points_world"].clone());
  c.insert("recorded_source_roles".into(),parse("[2,0,1]"));c.insert("recorded_source_parameters".into(),Json::Array(vec![Json::Number(fraction)]));
  c.insert("resolved_contact_fractions".into(),Json::Array(vec![Json::Number(fraction),Json::Other]));c.insert("resolved_contact_indexing".into(),Json::String("current_constraint_points_zero_based".into()));
  c.insert("native_plan".into(),Json::Array(center.into_iter().chain(normal).chain([4.]).map(Json::Number).collect()));
  let normal=crate::circle::Plan::new(center,normal,4.).unwrap().normal;
  c.insert("resolved_plan".into(),crate::prepared_result::map([("center",Json::Array(center.map(Json::Number).into())),("normal",Json::Array(normal.map(Json::Number).into())),("radius",Json::Number(4.)),("fit_deviation",Json::Number(0.)),("approx_deviation",Json::Number(0.))]));
  // Synthetic tangent circles are independently constructed from native D0/D1
  // oracle rows. This proves admission, not live solver root/family coverage.
  let admission=crate::circle_history::result::validate(crate::prepared_result::object(get(&v,"operation_context")).unwrap(),crate::prepared_result::object(get(&v,"snapshot")).unwrap());
  assert!(admission.is_ok(),"contact family {:?} parameter {:?}: {:?}",get(case,"name"),get(query,"parameter"),admission.err());count+=1;
 }}assert_eq!(count,183);
}
#[test]
fn ellipse_auxiliary_focus_results_retain_original_constructor_and_owned_vertex(){
 for(role,position)in [("ellipse_focus1","[6,20,30]"),("ellipse_focus2","[14,20,30]")]{
  let raw=focus_request(role,position);let result=parse(&assemble(&raw).expect("valid Ellipse focus result"));
  assert_eq!(get(get(&result,"operation_context"),"output_role"),&Json::String(role.into()));
  assert_eq!(get(get(&rows(get(&result,"input_snapshots"))[0],"geometry_data"),"points_xyz"),&parse("[[6,20,30],[14,20,30],[10,23,30]]"));
  assert_eq!(get(&rows(get(&result,"output_snapshots"))[0],"native_shape_type"),&Json::Number(7.));
 }
}
#[test]
fn focus_result_rejects_wrong_role_position_and_nonfocus_construction(){
 let raw=focus_request("ellipse_focus1","[6,20,30]");assert!(assemble(&raw).is_ok());
 for bad in [focus_request("ellipse_focus1","[14,20,30]"),focus_request("ellipse_focus3","[6,20,30]"),raw.replace("\"mark_foci\":true","\"mark_foci\":false"),raw.replace("\"native_shape_type\":7","\"native_shape_type\":5")]{
  assert_ne!(bad,raw);assert!(assemble(&bad).is_err());
 }
}
#[test]
fn focus_vertex_location_is_applied_once_and_root_location_must_agree(){
 let raw=focus_request("ellipse_focus1","[0,0,0]");
 let located=raw.replace("[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]","[[1,0,0,6],[0,1,0,20],[0,0,1,30],[0,0,0,1]]");
 assert!(assemble(&located).is_ok());
 let bad=located.replacen("\"native_root_location\":[[1,0,0,6]","\"native_root_location\":[[1,0,0,7]",1);assert_ne!(bad,located);assert!(assemble(&bad).is_err());
}
