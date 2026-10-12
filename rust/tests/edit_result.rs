// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::prepared_result::assemble;
fn snapshot(kind:u32,output:bool)->String {
 let representation=match kind{2=>"solid",5=>"polycurve",6=>"curve",_=>panic!("fixture")};
 let attributes=if output{r#"{"schema_version":1,"availability":"not_applicable_constructed_output"}"#}else{r#"{"schema_version":1,"document_uid":"doc-uuid","object_name":"Source","native_type":"Part::Feature","label":"Owned source","label2":null,"view_provider_type":null,"visibility":true,"selectable":true,"shape_appearance":null,"coverage":{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}"#};
 let solid=kind==2;
 format!(r#"{{"schema_version":1,"native_shape_type":{kind},"ownership":"private_native_copy","unit_context_snapshot":"","geometry_data":{{"representation":"{representation}","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 native fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[],"validation":{{"native_valid":true,"valid_solid":{solid}}}}},"source_attributes":{attributes}}}"#)
}
fn source(index:u32)->String {format!(r#"{{"document":"1","object":"{index}","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"doc-uuid","object_name":"Source","subelement":"","unit_context_snapshot":"","world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[]}}"#)}
fn request(kind:u32)->String {
 let fid=["","OM9-TOP11-008","OM9-TOP11-005","OM9-TOP11-010","OM9-SOLID-001","OM9-SOLID-002","OM9-SOLID-003","OM9-SOLID-004"][kind as usize];
 let operation=if kind>=4{"Boolean"}else{"NativeEdit"};let input=snapshot(if kind>=4{2}else if kind==2{5}else{6},false);
 let input_a=input.replace("\"object_name\":\"Source\"","\"object_name\":\"Source2\"");let input_b=input.replace("\"object_name\":\"Source\"","\"object_name\":\"Source3\"");
 let output=snapshot(if kind>=4{2}else if kind==1{5}else{6},true);let a=source(2).replace("\"object_name\":\"Source\"","\"object_name\":\"Source2\"");let b=source(3).replace("\"object_name\":\"Source\"","\"object_name\":\"Source3\"");let solid=kind>=4;
 let first=if matches!(kind,4|5|7){1}else{0};let owner=if kind==2{"0"}else{"null"};
 format!(r#"{{"snapshot":{output},"input_identity":{{"document_name":"fixture","name":"Edit"}},"validation":{{"native_valid":true,"valid_solid":{solid}}},"operation_context":{{"schema_version":1,"operation":"{operation}","feature_id":"{fid}","kind":{kind},"flags":4,"mode":0,"first_count":{first},"output_index":0,"output_count":1,"output_owner_index":{owner},"deleted_source_names":["Source2","Source3"],"tolerance_mm":0.01,"tolerance_report":null,"projection":null,"source_refs":[{a},{b}],"source_snapshots":[{input_a},{input_b}],"source_snapshot_frame":"document_world","output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"source_evidence_in_input_snapshots_output_attributes_unavailable"}}}}"#)
}
#[test]
fn edit_results_own_real_sources_for_all_seven_existing_cad_commands() {
 for kind in 1..=7 {
  let raw=assemble(&request(kind)).expect("existing Edit solver needs real result producer");
  assert!(raw.contains("Owned source"));assert!(raw.contains("18446744073709551615"));
  assert!(raw.contains(r#""input_snapshots":[{"#));assert!(raw.contains(r#""phase":"native_edit_construction""#));
  assert!(!raw.contains("upstream_archive_conversion"));assert!(!raw.contains("NativeBuilder"));
  assert!(raw.contains(r#""exactness":"uninspected""#));assert!(raw.contains(r#""topology_map":null"#));
 }
}
#[test]
fn inconsistent_edit_metadata_rejects_before_publication() {
 for (from,to) in [("OM9-TOP11-008","OM9-SOLID-003"),("\"flags\":4","\"flags\":0"),
  ("\"mode\":0","\"mode\":3"),("\"output_index\":0","\"output_index\":1"),
  ("\"output_count\":1","\"output_count\":0"),("\"first_count\":0","\"first_count\":3"),
  ("\"output_owner_index\":null","\"output_owner_index\":0"),
  ("\"tolerance_mm\":0.01","\"tolerance_mm\":0"),("\"projection\":null","\"projection\":{}"),
  ("\"generation\":\"3\"","\"generation\":\"0\""),
  ("\"source_snapshot_frame\":\"document_world\"","\"source_snapshot_frame\":\"object_local\""),
  ("\"availability\":\"not_applicable_constructed_output\"","\"availability\":\"unavailable_legacy_record\""),
  ("\"object_name\":\"Source2\"","\"object_name\":\"Other\""),
  ("\"source_refs\":[","\"source_refs\":[{},"),("\"source_snapshots\":[","\"source_snapshots\":[{},")]
 {assert!(assemble(&request(1).replacen(from,to,1)).is_err(),"{from}");}
 assert!(assemble(&request(4).replacen("\"first_count\":1","\"first_count\":0",1)).is_err());
 assert!(assemble(&request(2).replacen("\"output_owner_index\":0","\"output_owner_index\":2",1)).is_err());
 assert!(assemble(&request(6).replacen("\"valid_solid\":true","\"valid_solid\":false",1)).is_err());
}
#[test]
fn deleted_source_list_must_match_actual_command_policy() {
 assert!(assemble(&request(1).replace("\"deleted_source_names\":[\"Source2\",\"Source3\"]","\"deleted_source_names\":[]")).is_err());
 assert!(assemble(&request(4).replace("\"deleted_source_names\":[\"Source2\",\"Source3\"]","\"deleted_source_names\":[\"Source3\",\"Source2\"]")).is_err());
}
