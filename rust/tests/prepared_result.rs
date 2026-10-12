use openmatrix9_rust::prepared_result::*;
fn request(context:&str)->String {format!(r#"{{"snapshot":{{"schema_version":1,"native_shape_type":6,"ownership":"private_native_copy","unit_context_snapshot":"{context}","geometry_data":{{"representation":"curve","coordinates":"native_local_with_locations","native_master":{{"encoding":"occt_brep_ascii_v3","sdk_version":"8.0.0","payload":"CASCADE Topology V3 native fixture"}}}},"topology_data":{{"faces":[],"edges_3d":[],"vertices":[]}},"source_attributes":{{"name":"owned","layer":"fixture","locked":false,"visible":true,"tolerance":0.000001,"color":[20,40,60]}}}},"input_identity":{{"document_name":"fixture","name":"owned"}},"validation":{{"native_valid":true,"valid_solid":false}}}}"#)}
fn builder_request()->String {
 let attrs=r#"{"schema_version":1,"document_uid":"doc-uuid","object_name":"Seed","native_type":"Part::Feature","label":"Owned seed","label2":null,"view_provider_type":null,"visibility":false,"selectable":true,"shape_appearance":null,"coverage":{"other_app_properties":"uninspected","rhino_user_data":"uninspected"}}"#;
 let context=r#"{"schema_version":1,"operation":"NativeBuilder","feature_id":"OM9-SETTING-001","recipe":"{\"schema\":1,\"evaluator\":\"om9.affine-template\",\"version\":1,\"settings\":{}}","scale_to_gem":true,"template_slot":0,"template_count":1,"initial_dimensions":[1,2,4],"current_dimensions":[2,2,4],"affine":[2,1,1,0,0,0],"source_native_shape_type":2,"template_native_shape_type":6,"source_ref":{"document":"1","object":"2","generation":"3","revision":"18446744073709551615","epoch":"4","document_uid":"doc-uuid","object_name":"Gem","unit_context_snapshot":"","world":[1,0,0,103,0,1,0,0,0,0,1,0,0,0,0,1],"ancestors":[]},"output_parent_to_world":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"attribute_role":"immutable_template_source_evidence"}"#;
 request("").replace(r#"{"name":"owned","layer":"fixture","locked":false,"visible":true,"tolerance":0.000001,"color":[20,40,60]}"#,attrs)
  .replacen("{",&format!("{{\"operation_context\":{context},"),1)
}
fn builder_observations()->String {
 let source=builder_request().replace(r#""vertices":[]"#,r#""vertices":[{"native_topology_index":"1"}]"#)
  .replace(r#""topology_data":{"faces""#,r#""topology_data":{"validation":{"native_valid":true,"valid_solid":false},"faces""#);
 let begin=source.find(r#""snapshot":"#).unwrap()+11;
 let end=source.find(",\"input_identity\"").unwrap();
 let snapshot=&source[begin..end];
 let observation=format!(r#"{{"schema_version":1,"method":"composed_occt_modified_shape","input_snapshot":{snapshot},"pairs":[{{"kind":"vertex","source_index":1,"output_index":1,"samples":[{{"source_point":[3,0,0],"output_point":[109,0,0]}}]}}]}}"#);
 source.replacen(r#""attribute_role":"immutable_template_source_evidence""#,&format!(r#""attribute_role":"immutable_template_source_evidence","native_transform_observations":{observation}"#),1)
}
#[test]
fn native_transform_observations_produce_measured_residual_and_real_correspondence(){
 let output=assemble(&builder_observations()).unwrap();
 for field in [r#""measurement":"native_point_samples""#,r#""max_distance":0"#,r#""complete":true"#,r#""input_snapshots":["#]{assert!(output.contains(field),"{field}");}
 assert!(!output.contains(r#""exactness":"exact""#));
}
#[test]
fn inconsistent_native_observations_cannot_certify_or_publish_the_output(){
 let source=builder_observations();
 for (before,after) in [(r#""output_point":[109,0,0]"#,r#""output_point":[110,0,0]"#),(r#""source_index":1"#,r#""source_index":2"#),(r#""schema_version":1,"method":"composed_occt_modified_shape""#,r#""schema_version":2,"method":"composed_occt_modified_shape""#)]{
  assert!(assemble(&source.replace(before,after)).is_err(),"{before}");
 }
}
#[test]
fn correspondence_must_name_the_actual_snapshot_namespace(){
 let source=builder_observations();
 // Count equality alone cannot certify correspondence to an absent native ID.
 for row in [r#"{"native_topology_index":"77"}"#,r#"{}"#,r#"{"native_topology_index":"0"}"#,r#"{"native_topology_index":1}"#] {
  assert!(assemble(&source.replacen(r#"{"native_topology_index":"1"}"#,row,1)).is_err(),"{row}");
 }
}
#[test]
fn unavailable_native_query_is_owned_explicit_and_cannot_claim_completion(){
 let request=builder_observations().replace(r#""edges_3d":[]"#,r#""edges_3d":[{"native_topology_index":"1"}]"#)
  .replace(r#""samples":[{"source_point":[3,0,0],"output_point":[109,0,0]}]}]"#,
    r#""samples":[{"source_point":[3,0,0],"output_point":[109,0,0]}]},{"kind":"edge","source_index":1,"output_index":1,"source_domain":[0,1],"output_domain":[0,1],"samples":[],"query_failures":[{"source_parameter":0.3125,"source_point":[0.3125,0,0],"error":"KernelFailure","reason":"no_projection_solution"}]}]"#);
 let result=assemble(&request).unwrap();
 assert!(result.contains(r#""recorded_queries_complete":false"#));
 assert!(result.contains(r#""unavailable_queries":[{"#));
 assert!(result.contains(r#""reason":"no_projection_solution"#));
 assert!(assemble(&request.replace(r#""source_parameter":0.3125"#,r#""source_parameter":2"#)).is_err());
 assert!(assemble(&request.replace("KernelFailure","success")).is_err());
}
#[test]
fn unavailable_surface_query_retains_uv_and_rejects_invalid_failure_evidence(){
 let request=builder_observations().replace(r#""faces":[]"#,r#""faces":[{"native_topology_index":"1"}]"#)
  .replace(r#""native_shape_type":6"#,r#""native_shape_type":4"#).replace(r#""template_native_shape_type":6"#,r#""template_native_shape_type":4"#)
  .replace(r#""representation":"curve""#,r#""representation":"trimmed_face""#)
  .replace(r#""samples":[{"source_point":[3,0,0],"output_point":[109,0,0]}]}]"#,
    r#""samples":[{"source_point":[3,0,0],"output_point":[109,0,0]}]},{"kind":"face","source_index":1,"output_index":1,"source_uv_domain":[0,1,0,1],"output_uv_domain":[0,1,0,1],"samples":[],"query_failures":[{"source_uv":[0.5,0.5],"source_point":[3,0,0],"error":"KernelFailure","reason":"native_projection_failure"}]}]"#);
 let result=assemble(&request).unwrap();
 assert!(result.contains(r#""recorded_queries_complete":false"#));
 assert!(result.contains(r#""source_uv":[0.5,0.5]"#));
 assert!(result.contains(r#""kind":"face""#));
 assert!(assemble(&request.replace(r#""source_uv":[0.5,0.5]"#,r#""source_uv":[0.5,2]"#)).is_err());
 assert!(assemble(&request.replace("native_projection_failure","success")).is_err());
}
#[test]
fn native_face_uv_queries_are_retained_and_reject_out_of_domain_parameters(){
 let face=builder_observations().replace(r#""faces":[],"edges_3d":[],"vertices":[{"native_topology_index":"1"}]"#,r#""faces":[{"native_topology_index":"1"}],"edges_3d":[],"vertices":[]"#)
  .replace(r#""native_shape_type":6"#,r#""native_shape_type":4"#).replace(r#""template_native_shape_type":6"#,r#""template_native_shape_type":4"#)
  .replace(r#""representation":"curve""#,r#""representation":"trimmed_face""#)
  .replace(r#""kind":"vertex""#,r#""kind":"face","source_uv_domain":[0,1,0,1],"output_uv_domain":[0,1,0,1]"#)
  .replace(r#""source_point":[3,0,0]"#,r#""source_uv":[0.5,0.5],"output_uv":[0.5,0.5],"source_point":[3,0,0]"#);
 let result=assemble(&face).unwrap();assert!(result.contains(r#""face_queries":[{"#));
 assert!(assemble(&face.replace(r#""source_uv":[0.5,0.5]"#,r#""source_uv":[2,0.5]"#)).is_err());
}
#[test]
fn builder_result_owns_actual_operation_recipe_source_and_attributes(){
 let mut input=builder_request();let result=assemble(&input).unwrap();input.clear();
 for expected in [r#""operation":"NativeBuilder""#,r#""lifetime":"immutable_native_operation_evidence_not_current_validity""#,r#""revision":"18446744073709551615""#,r#""object_name":"Seed""#,r#""visibility":false"#,r#""affine":[2,1,1,0,0,0]"#] {assert!(result.contains(expected),"{expected}");}
 assert!(!result.contains("upstream_archive_conversion"));
}
#[test]
fn native_context_cannot_be_ignored_or_certify_an_inconsistent_affine_recipe(){
 // Even an import-shaped request must reject an explicit unknown operation.
 let import=request("").replacen("{",r#"{"operation_context":{"operation":"future"},"#,1);
 assert!(assemble(&import).is_err());
 let source=builder_request();
 for (before,after) in [(r#""affine":[2,1,1,0,0,0]"#,r#""affine":[1,1,1,0,0,0]"#),(r#""template_slot":0"#,r#""template_slot":1"#),(r#""generation":"3""#,r#""generation":"0""#),(r#""current_dimensions":[2,2,4]"#,r#""current_dimensions":[0,2,4]"#),(r#""visibility":false"#,r#""visibility":"false""#)] {assert!(assemble(&source.replace(before,after)).is_err(),"{before}");}
}
#[test]
fn persisted_attributes_are_bounded_slot_aligned_and_reject_uninspected_inventions(){unsafe{
 let request=builder_request();let begin=request.find(r#""source_attributes":"#).unwrap()+20;
 let end=request[begin..].find("}},\"input_identity\"").unwrap()+begin+1;
 let attrs=format!("[{}]",&request[begin..end]);
 assert_eq!(validate_template_attributes(&attrs,1),Ok(()));
 assert!(validate_template_attributes(&attrs,2).is_err());
 for raw in [attrs.replace(r#""visibility":false"#,r#""visibility":42"#),attrs.replace("uninspected","certified"),"[{}]".into()] {
  assert!(validate_template_attributes(&raw,1).is_err());
 }
 assert_eq!(validate_template_attributes("",1),Ok(()));
 assert!(validate_template_attributes("",0).is_err());
 assert!(validate_template_attributes(&" ".repeat(MAX_RESULT_BYTES+1),1).is_err());
 assert_eq!(om9_builder_attributes_validate(attrs.as_ptr(),attrs.len(),1),0);
 assert_ne!(om9_builder_attributes_validate(std::ptr::null(),1,1),0);
 assert_ne!(om9_builder_attributes_validate((usize::MAX-1)as *const u8,4,1),0);
 assert_eq!(om9_builder_attributes_validate(std::ptr::null(),0,1),0);
}}
#[test]
fn result_owns_complete_snapshot_schema_and_unknown_conversion_fields(){
 let mut input=request("");let result=assemble(&input).unwrap();input.clear();
 for expected in ["\"output_snapshots\":[","\"unit_context_revision\":null","\"geometry_data\":","\"topology_data\":","\"source_attributes\":","\"from\":","\"to\":\"curve\"","\"exactness\":\"uninspected\"","\"topology_map\":null"] {assert!(result.contains(expected),"{expected}");}
 assert!(result.contains("CASCADE Topology V3 native fixture"));assert!(!result.contains("\"parameterization_preserved\":true"));
}
#[test]
fn explicit_context_uses_existing_decoder_and_preserves_full_u64_revision(){
 let result=assemble(&request("1|18446744073709551615|2.54e1|1e0|1e-6|1e-6|1e-3")).unwrap();
 assert!(result.contains("\"unit_context_revision\":\"18446744073709551615\""));
 for context in ["future","1|0|1|1|1e-6|1e-6|1e-3","1|1|-1|1|1e-6|1e-6|1e-3"] {assert!(assemble(&request(context)).is_err());}
}
#[test]
fn malformed_or_unadmitted_native_snapshot_never_produces_acceptance(){
 let source=request("");
 for (before,after) in [("\"schema_version\":1","\"schema_version\":2"),("\"native_valid\":true","\"native_valid\":false"),("\"native_shape_type\":6","\"native_shape_type\":2"),("\"representation\":\"curve\"","\"representation\":\"solid\""),("\"faces\":[]","\"faces\":null"),("occt_brep_ascii_v3","unknown_encoding"),("\"color\":[20,40,60]","\"color\":[20,40]"),("\"tolerance\":0.000001","\"tolerance\":0")]{assert!(assemble(&source.replace(before,after)).is_err(),"{before}");}
}
#[test]
fn owned_json_is_bounded_and_rejects_trailing_or_duplicate_fields(){
 assert!(assemble("{}").is_err());assert!(assemble(&(request("")+" null")).is_err());
 assert!(assemble(&request("").replace("\"native_shape_type\":6","\"native_shape_type\":6,\"native_shape_type\":2")).is_err());
 assert_eq!(assemble(&" ".repeat(MAX_RESULT_BYTES+1)),Err(2));
 assert!(assemble(&request("").replace("native fixture",&"x".repeat(MAX_MASTER_BYTES))).is_err());
}
#[test]
fn synchronous_abi_queries_copies_and_rejects_invalid_bounds_without_retaining_pointers(){unsafe{
 let input=request("");let query=om9_prepared_result_json(input.as_ptr(),input.len(),std::ptr::null_mut(),0);
 assert_eq!(query.status,0);assert_eq!(query.reserved,0);assert!(query.needed>input.len());
 let mut output=vec![0u8;query.needed];let copied=om9_prepared_result_json(input.as_ptr(),input.len(),output.as_mut_ptr(),output.len());
 assert_eq!(copied.status,0);assert_eq!(copied.needed,query.needed);assert_eq!(String::from_utf8(output).unwrap(),assemble(&input).unwrap());
 assert_ne!(om9_prepared_result_json(input.as_ptr(),input.len(),std::ptr::null_mut(),1).status,0);
 let mut short=[9u8;3];assert_ne!(om9_prepared_result_json(input.as_ptr(),input.len(),short.as_mut_ptr(),short.len()).status,0);assert_eq!(short,[9;3]);
 assert_ne!(om9_prepared_result_json(std::ptr::null(),0,std::ptr::null_mut(),0).status,0);
 assert_ne!(om9_prepared_result_json(input.as_ptr(),MAX_RESULT_BYTES+1,std::ptr::null_mut(),0).status,0);
}}
