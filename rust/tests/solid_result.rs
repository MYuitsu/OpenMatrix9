// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::{prepared_result::{assemble,om9_prepared_result_json},solid::{Session,Kind}};
fn attrs()->&'static str{r#"{"schema_version":1,"document_uid":"uid","object_name":"Source","native_type":"Part::Feature","label":"Owned source","label2":null,"view_provider_type":null,"visibility":true,"selectable":true,"shape_appearance":null,"coverage":{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}"#}
fn snapshot(kind:u32,attributes:&str,solid:bool)->String{
 let representation=match kind{2=>"solid",6=>"curve",_=>"point"};let faces=if solid{r#"[{"native_topology_index":"1"}]"#}else{"[]"};
 format!(r#"{{"schema_version":1,"native_shape_type":{kind},"ownership":"private_native_copy","unit_context_snapshot":"","geometry_data":{{"representation":"{representation}","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 unit fixture"}}}},"topology_data":{{"faces":{faces},"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":{solid},"closed":{solid}}}}},"source_attributes":{attributes}}}"#)
}
fn construction(kind:Kind,inputs:&[&str])->String{
 let mut s=Session::new(kind);for input in inputs{s.input(input).unwrap();}s.construction_evidence(u64::MAX).unwrap()
}
fn source(role:&str,index:usize,begin:usize,end:usize)->String{
 format!(r#"{{"document":"1","object":"2","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"uid","object_name":"Source","subelement":"Edge1","unit_context_snapshot":"","world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[],"role":"{role}","role_index":{index},"point_begin":{begin},"point_end":{end},"snapshot_frame":"document_world"}}"#)
}
fn request(recipe:&str,refs:&str,snapshots:&str)->String{
 let id=if recipe.contains("OM9-SOLID-012"){"OM9-SOLID-012"}else{"OM9-SOLID-014"};
 let output=snapshot(2,r#"{"schema_version":1,"availability":"not_applicable_constructed_output"}"#,true);
 format!(r#"{{"snapshot":{output},"validation":{{"native_valid":true,"valid_solid":true,"closed":true}},"input_identity":{{"document_name":"fixture","name":"Solid"}},"operation_context":{{"schema_version":1,"operation":"ConstructSolid","feature_id":"{id}","construction":{recipe},"source_refs":[{refs}],"source_snapshots":[{snapshots}],"output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"source_evidence_in_input_snapshots_output_attributes_unavailable"}}}}"#)
}
fn box_request()->String{request(&construction(Kind::Box,&["Center","10,20,30","4","6","8"]),"","")}
#[test]
fn solid_result_uses_its_real_operation_owned_numeric_input_and_solid_master(){
 let output=assemble(&box_request()).expect("completed Solid must use the shared producer");
 for expected in [r#""operation":"ConstructSolid""#,r#""representation":"point_cloud""#,r#""points_xyz":[[10,20,30]]"#,"18446744073709551615","CASCADE Topology V3",r#""phase":"native_solid_construction""#,r#""exactness":"uninspected""#,r#""topology_map":null"#]{assert!(output.contains(expected),"{expected}");}
 assert!(!output.contains("NativeBuilder")&&!output.contains("upstream_archive_conversion"));
}
#[test]
fn around_curve_result_binds_the_actual_owned_curve_source(){
 let mut s=Session::new(Kind::Sphere);s.input("AroundCurve").unwrap();
 s.accept_reference([7.,8.,9.],[0.,0.,1.]).unwrap();s.accept_reference([1.,2.,3.],[0.,0.,1.]).unwrap();s.input("Radius=2").unwrap();
 let recipe=s.construction_evidence(7).unwrap();let refs=source("path",0,0,0);let input=snapshot(6,attrs(),false);
 let raw=request(&recipe,&refs,&input);let result=assemble(&raw).expect("owned native curve input");
 assert!(result.contains("Owned source")&&result.contains(r#""points_xyz":[[7,8,9],[1,2,3]]"#));
 for bad in [request(&recipe,"",""),raw.replacen("\"object_name\":\"Source\"","\"object_name\":\"Other\"",1),raw.replacen("\"role\":\"path\"","\"role\":\"fit_points\"",1),raw.replacen("\"generation\":\"3\"","\"generation\":\"0\"",1),raw.replacen("\"unit_context_snapshot\":\"\"","\"unit_context_snapshot\":\"1|1|1e0|1e0|1e-2|1e-3|1e-2\"",1)]{assert!(assemble(&bad).is_err());}
}
#[test]
fn selected_fit_point_ranges_preserve_batch_provenance_and_reject_wrong_bindings(){
 let recipe=construction(Kind::Sphere,&["FitPoints","1,0,0","0,1,0","-1,0,0",""]);
 let refs=source("fit_points",0,0,3);
 let input=format!(r#"{{"schema_version":1,"ownership":"private_plain_copy","unit_context_snapshot":"","source_attributes":{},"geometry_data":{{"kind":"point_cloud","points_xyz":[[1,0,0],[0,1,0],[-1,0,0]],"triangles":null,"other_native_channels":"uninspected"}},"topology_data":{{}},"native_discrete_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"native_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"selected_points_world":[[1,0,0],[0,1,0],[-1,0,0]]}}"#,attrs());
 let raw=request(&recipe,&refs.replace("document_world","native_local_with_locations"),&input);
 assert!(assemble(&raw).is_ok());
 for bad in [raw.replacen("\"point_end\":3","\"point_end\":4",1),raw.replacen("\"point_begin\":0","\"point_begin\":1",1),raw.replacen("\"selected_points_world\":[[1,0,0]","\"selected_points_world\":[[2,0,0]",1),raw.replacen("\"role_index\":0","\"role_index\":1",1)]{assert!(assemble(&bad).is_err());}
}
#[test]
fn completed_fit_and_tangent_recipes_cannot_drop_normalized_constraints(){
 let recipe=construction(Kind::Sphere,&["FitPoints","1,0,0","0,1,0","-1,0,0",""]);
 assert!(assemble(&request(&recipe,"","")).is_ok());
 let corrupt=recipe.replacen("\"points\":[[1,0,0],[0,1,0],[-1,0,0]]","\"points\":[]",1);
 assert!(assemble(&request(&corrupt,"","")).is_err(),"empty normalized fit input must not be accepted");
 let mut s=Session::new(Kind::Sphere);s.input("Tangent").unwrap();
 for p in [[1.,0.,0.],[0.,1.,0.],[-1.,0.,0.]]{s.input("Point").unwrap();s.point(p).unwrap();}
 s.resolve([0.,0.,0.],1.).unwrap();let recipe=s.construction_evidence(12).unwrap();
 assert!(assemble(&request(&recipe,"","")).is_ok());
 let corrupt=recipe.replacen("\"points\":[[1,0,0],[0,1,0],[-1,0,0]]","\"points\":[]",1)
  .replacen("\"constraint_types\":[true,true,true]","\"constraint_types\":[]",1);
 assert!(assemble(&request(&corrupt,"","")).is_err(),"completed Tangent cannot drop all constraints");
}
#[test]
fn repeated_source_ranges_share_an_owned_snapshot_with_explicit_indices(){
 let recipe=construction(Kind::Sphere,&["FitPoints","0,1,0","-1,0,0","1,0,0","1,0,0",""]);
 let indexed=|r:String|format!("{},\"input_snapshot_index\":1}}",r.strip_suffix('}').unwrap());
 let refs=format!("{},{}",indexed(source("fit_points",0,2,3)),indexed(source("fit_points",1,3,4)));
 let cad=snapshot(6,attrs(),false);
 let input=format!("{},\"selected_points_world\":[[1,0,0]]}}",cad.strip_suffix('}').unwrap());
 let raw=request(&recipe,&refs,&input).replacen("\"source_refs\":[","\"source_snapshot_indexing\":\"input_snapshots_zero_based_including_numeric_input\",\"source_refs\":[",1);
 let result=assemble(&raw).expect("repeated references must preserve all ranges without duplicating the native master");
 assert_eq!(result.matches("CASCADE Topology V3 unit fixture").count(),2,"one source master and one output master");
 for bad in [raw.replacen("\"input_snapshot_index\":1","\"input_snapshot_index\":0",1),raw.replacen("\"input_snapshot_index\":1","\"input_snapshot_index\":2",1),raw.replacen(",\"input_snapshot_index\":1","",1)]{assert!(assemble(&bad).is_err(),"missing/out-of-range mapping cannot alias a source");}
}
#[test]
fn around_curve_preserves_the_supported_radius_point_flow(){
 let mut s=Session::new(Kind::Sphere);s.input("AroundCurve").unwrap();
 s.accept_reference([7.,8.,9.],[0.,0.,1.]).unwrap();s.accept_reference([1.,2.,3.],[0.,0.,1.]).unwrap();
 s.point([3.,2.,3.]).unwrap();
 let recipe=s.construction_evidence(13).unwrap();
 assert!(assemble(&request(&recipe,&source("path",0,0,0),&snapshot(6,attrs(),false))).is_ok(),"radius point is an existing native command flow");
}
#[test]
fn completed_four_point_recipe_retains_the_fourth_construction_point(){
 let recipe=construction(Kind::Sphere,&["4Point","1,0,0","0,1,0","-1,0,0","0,0,1"]);
 assert!(assemble(&request(&recipe,"","")).is_ok());
 let corrupt=recipe.replacen("\"accepted_points\":[[1,0,0],[0,1,0],[-1,0,0],[0,0,1]]","\"accepted_points\":[[1,0,0],[0,1,0],[-1,0,0]]",1);
 assert_ne!(corrupt,recipe,"witness must actually delete the fourth accepted point");
 assert!(assemble(&request(&corrupt,"","")).is_err(),"completed 4Point cannot lose its fourth construction input");
}
#[test]
fn discrete_source_master_is_bound_to_the_accepted_world_batch(){
 let recipe=construction(Kind::Sphere,&["FitPoints","1,0,0","0,1,0","-1,0,0",""]);
 let refs=source("fit_points",0,0,3).replace("document_world","native_local_with_locations");
 let input=format!(r#"{{"schema_version":1,"ownership":"private_plain_copy","unit_context_snapshot":"","source_attributes":{},"geometry_data":{{"kind":"point_cloud","points_xyz":[[1,0,0],[0,1,0],[-1,0,0]],"triangles":null,"other_native_channels":"uninspected"}},"topology_data":{{}},"native_discrete_location":[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]],"native_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"selected_points_world":[[1,0,0],[0,1,0],[-1,0,0]]}}"#,attrs());
 let raw=request(&recipe,&refs,&input);assert!(assemble(&raw).is_ok());
 let corrupt=raw.replacen("\"kind\":\"point_cloud\",\"points_xyz\":[[1,0,0]","\"kind\":\"point_cloud\",\"points_xyz\":[[2,0,0]",1);
 assert!(assemble(&corrupt).is_err(),"selected batch must derive from the owned native geometry");
 let corrupt=raw.replacen("\"native_discrete_location\":[[1,0,0,0]","\"native_discrete_location\":[[1,0,0,1]",1);
 assert!(assemble(&corrupt).is_err(),"native location must agree with source world and selected points");
}
#[test]
fn malformed_recipe_frames_or_non_solid_output_never_receive_acceptance(){
 let raw=box_request();
 for (from,to) in [("\"kind\":\"Box\"","\"kind\":\"Sphere\""),("\"mode\":\"Center\"","\"mode\":\"FitPoints\""),("\"phase\":6","\"phase\":4"),("\"generation\":\"18446744073709551615\"","\"generation\":\"0\""),("\"coordinates\":\"document_world_mm\"","\"coordinates\":\"native_local\""),("\"native_shape_type\":2","\"native_shape_type\":3"),("\"valid_solid\":true","\"valid_solid\":false"),("\"closed\":true","\"closed\":false"),("\"output_parent_to_world\":[1,0,0,0","\"output_parent_to_world\":[0,0,0,0")]{assert!(assemble(&raw.replacen(from,to,1)).is_err(),"{from}");}
}
#[test]
fn shared_solid_result_copy_owns_bytes_and_rejects_without_touching_canaries(){
 let raw=box_request();unsafe{
  let query=om9_prepared_result_json(raw.as_ptr(),raw.len(),std::ptr::null_mut(),0);assert_eq!(query.status,0);assert!(query.needed>100);
  let mut output=vec![0xA5;query.needed+16];let short=om9_prepared_result_json(raw.as_ptr(),raw.len(),output.as_mut_ptr(),1);assert_eq!(short.status,2);assert!(output.iter().all(|&b|b==0xA5));
  let copied=om9_prepared_result_json(raw.as_ptr(),raw.len(),output.as_mut_ptr(),query.needed);assert_eq!(copied.status,0);assert!(output[query.needed..].iter().all(|&b|b==0xA5));let owned=output[..query.needed].to_vec();
  let invalid=raw.replacen("\"phase\":6","\"phase\":4",1);output.fill(0xA5);assert_ne!(om9_prepared_result_json(invalid.as_ptr(),invalid.len(),output.as_mut_ptr(),output.len()).status,0);assert!(output.iter().all(|&b|b==0xA5));assert!(std::str::from_utf8(&owned).unwrap().contains("ConstructSolid"));
 }
}
